#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct Property {
    id: u64,
    address: String,
    price: f64,
    description: String,
    owner: Option<String>,
}

impl Storable for Property {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Property {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static PROPERTY_ID: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static PROPERTY_STORAGE: RefCell<StableBTreeMap<u64, Property, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))))
    );
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

#[derive(candid::CandidType, Serialize, Deserialize)]
struct PropertyPayload {
    address: String,
    price: f64,
    description: String,
}

#[ic_cdk::query]
fn get_properties() -> Result<Vec<Property>, Error> {
    let properties = PROPERTY_STORAGE
        .with(|m| m.borrow().iter().map(|(_, v)| v.clone()).collect::<Vec<_>>());
    if properties.len() == 0 {
        return Err(Error::NotFound {
            msg: "No properties found".to_string(),
        });
    }
    Ok(properties)
}

#[ic_cdk::query]
fn get_property_by_id(id: u64) -> Result<Property, Error> {
    PROPERTY_STORAGE.with(|service| {
        service
            .borrow_mut()
            .get(&id)
            .ok_or(Error::NotFound {
                msg: format!("Property with id={} not found", id),
            })
    })
}

#[ic_cdk::update]
fn add_property(payload: PropertyPayload) -> Result<Property, Error> {
    let id = PROPERTY_ID
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    let property = Property {
        id,
        address: payload.address,
        price: payload.price,
        description: payload.description,
        owner: None, // Initially, no owner
    };

    PROPERTY_STORAGE.with(|m| m.borrow_mut().insert(id, property.clone()));
    Ok(property)
}

#[ic_cdk::update]
fn update_property(id: u64, payload: PropertyPayload) -> Result<Property, Error> {
    PROPERTY_STORAGE.with(|m| {
        let mut property = m
            .borrow_mut()
            .get(&id)
            .ok_or(Error::NotFound {
                msg: format!("Property with id={} not found", id),
            })?;

        property.address = payload.address;
        property.price = payload.price;
        property.description = payload.description;

        m.borrow_mut().insert(id, property.clone());
        Ok(property)
    })
}

#[ic_cdk::update]
fn delete_property(id: u64) -> Result<Property, Error> {
    PROPERTY_STORAGE.with(|m| {
        m.borrow_mut()
            .remove(&id)
            .ok_or(Error::NotFound {
                msg: format!("Property with id={} not found", id),
            })
    })
}

#[ic_cdk::update]
fn transfer_property_ownership(id: u64, new_owner: String) -> Result<Property, Error> {
    PROPERTY_STORAGE.with(|m| {
        let mut property = m
            .borrow_mut()
            .get(&id)
            .ok_or(Error::NotFound {
                msg: format!("Property with id={} not found", id),
            })?;

        property.owner = Some(new_owner.clone());

        m.borrow_mut().insert(id, property.clone());
        Ok(property)
    })
}

ic_cdk::export_candid!();