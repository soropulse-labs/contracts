#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, Env, Map, String,
};

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_TITLE_BYTES: u32 = 128;
pub const MAX_CAPACITY: u32 = 128;
const TTL_THRESHOLD: u32 = 100_000;
const TTL_EXTEND_TO: u32 = 500_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TicketEvent {
    pub id: u64,
    pub title: String,
    pub capacity: u32,
    pub reserved: u32,
    pub closed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reservation {
    pub id: u64,
    pub event_id: u64,
    pub attendee: Address,
}

// The event record and attendee registry are one persistent ledger entry. They
// expire and restore together, so a lost reservation cannot silently free a seat.
#[contracttype]
#[derive(Clone)]
struct EventRecord {
    info: TicketEvent,
    attendees: Map<Address, u64>,
}

#[contracttype]
#[derive(Clone)]
enum Key {
    Organizer,
    LastEventId,
    NextReservationId,
    Event(u64),
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    Unauthorized = 1,
    EventNotFound = 2,
    DuplicateEventId = 3,
    InvalidCapacity = 4,
    InvalidTitle = 5,
    ReservationsClosed = 6,
    CapacityReached = 7,
    AlreadyReserved = 8,
    ArithmeticOverflow = 9,
}

#[contractevent(topics = ["created"])]
pub struct EventCreated {
    #[topic]
    pub schema_version: u32,
    #[topic]
    pub event_id: u64,
    pub title: String,
    pub capacity: u32,
}

#[contractevent(topics = ["ticket"])]
pub struct TicketReserved {
    #[topic]
    pub schema_version: u32,
    #[topic]
    pub event_id: u64,
    #[topic]
    pub reservation_id: u64,
    pub attendee: Address,
    pub reserved_after: u32,
}

#[contractevent(topics = ["closed"])]
pub struct EventClosed {
    #[topic]
    pub schema_version: u32,
    #[topic]
    pub event_id: u64,
    pub reserved: u32,
}

#[contract]
pub struct TicketReservation;

fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

fn extend_event(env: &Env, event_id: u64) {
    env.storage()
        .persistent()
        .extend_ttl(&Key::Event(event_id), TTL_THRESHOLD, TTL_EXTEND_TO);
}

fn organizer(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&Key::Organizer)
        .ok_or(Error::Unauthorized)
}

fn require_organizer(env: &Env, caller: &Address) -> Result<(), Error> {
    if caller != &organizer(env)? {
        return Err(Error::Unauthorized);
    }
    caller.require_auth();
    Ok(())
}

fn record(env: &Env, event_id: u64) -> Result<EventRecord, Error> {
    env.storage()
        .persistent()
        .get(&Key::Event(event_id))
        .ok_or(Error::EventNotFound)
}

#[contractimpl]
impl TicketReservation {
    /// Deployment-time initialization. The organizer address must authorize
    /// the deployment invocation and cannot be replaced.
    pub fn __constructor(env: Env, organizer: Address) {
        organizer.require_auth();
        env.storage().instance().set(&Key::Organizer, &organizer);
        env.storage().instance().set(&Key::LastEventId, &0_u64);
        env.storage()
            .instance()
            .set(&Key::NextReservationId, &1_u64);
        extend_instance(&env);
    }

    pub fn schema_version(_env: Env) -> u32 {
        SCHEMA_VERSION
    }

    pub fn get_organizer(env: Env) -> Result<Address, Error> {
        organizer(&env)
    }

    /// IDs must increase strictly. This high-water mark prevents reuse of an
    /// archived event ID even when its persistent record is unavailable.
    pub fn create_event(
        env: Env,
        caller: Address,
        event_id: u64,
        title: String,
        capacity: u32,
    ) -> Result<(), Error> {
        require_organizer(&env, &caller)?;
        if capacity == 0 || capacity > MAX_CAPACITY {
            return Err(Error::InvalidCapacity);
        }
        if title.is_empty() || title.len() > MAX_TITLE_BYTES {
            return Err(Error::InvalidTitle);
        }
        let last: u64 = env
            .storage()
            .instance()
            .get(&Key::LastEventId)
            .ok_or(Error::ArithmeticOverflow)?;
        if event_id <= last {
            return Err(Error::DuplicateEventId);
        }
        let info = TicketEvent {
            id: event_id,
            title: title.clone(),
            capacity,
            reserved: 0,
            closed: false,
        };
        env.storage().persistent().set(
            &Key::Event(event_id),
            &EventRecord {
                info,
                attendees: Map::new(&env),
            },
        );
        env.storage().instance().set(&Key::LastEventId, &event_id);
        extend_instance(&env);
        extend_event(&env, event_id);
        EventCreated {
            schema_version: SCHEMA_VERSION,
            event_id,
            title,
            capacity,
        }
        .publish(&env);
        Ok(())
    }

    pub fn reserve(env: Env, event_id: u64, attendee: Address) -> Result<Reservation, Error> {
        attendee.require_auth();
        let mut event = record(&env, event_id)?;
        if event.attendees.contains_key(attendee.clone()) {
            return Err(Error::AlreadyReserved);
        }
        if event.info.closed {
            return Err(Error::ReservationsClosed);
        }
        if event.info.reserved >= event.info.capacity {
            return Err(Error::CapacityReached);
        }
        let reserved_after = event
            .info
            .reserved
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        let id: u64 = env
            .storage()
            .instance()
            .get(&Key::NextReservationId)
            .ok_or(Error::ArithmeticOverflow)?;
        let next = id.checked_add(1).ok_or(Error::ArithmeticOverflow)?;
        event.attendees.set(attendee.clone(), id);
        event.info.reserved = reserved_after;
        env.storage()
            .persistent()
            .set(&Key::Event(event_id), &event);
        env.storage().instance().set(&Key::NextReservationId, &next);
        extend_instance(&env);
        extend_event(&env, event_id);
        TicketReserved {
            schema_version: SCHEMA_VERSION,
            event_id,
            reservation_id: id,
            attendee: attendee.clone(),
            reserved_after,
        }
        .publish(&env);
        Ok(Reservation {
            id,
            event_id,
            attendee,
        })
    }

    /// Closing twice succeeds and emits no second event.
    pub fn close_event(env: Env, caller: Address, event_id: u64) -> Result<(), Error> {
        require_organizer(&env, &caller)?;
        let mut event = record(&env, event_id)?;
        if !event.info.closed {
            event.info.closed = true;
            env.storage()
                .persistent()
                .set(&Key::Event(event_id), &event);
            EventClosed {
                schema_version: SCHEMA_VERSION,
                event_id,
                reserved: event.info.reserved,
            }
            .publish(&env);
        }
        extend_instance(&env);
        extend_event(&env, event_id);
        Ok(())
    }

    pub fn get_event(env: Env, event_id: u64) -> Result<TicketEvent, Error> {
        Ok(record(&env, event_id)?.info)
    }

    pub fn get_reservation(
        env: Env,
        event_id: u64,
        attendee: Address,
    ) -> Result<Option<Reservation>, Error> {
        let event = record(&env, event_id)?;
        Ok(event.attendees.get(attendee.clone()).map(|id| Reservation {
            id,
            event_id,
            attendee,
        }))
    }

    pub fn has_reserved(env: Env, event_id: u64, attendee: Address) -> Result<bool, Error> {
        Ok(record(&env, event_id)?.attendees.contains_key(attendee))
    }

    /// Anyone can refresh the TTL of a live event and its contract instance.
    pub fn refresh_event(env: Env, event_id: u64) -> Result<(), Error> {
        record(&env, event_id)?;
        extend_instance(&env);
        extend_event(&env, event_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
