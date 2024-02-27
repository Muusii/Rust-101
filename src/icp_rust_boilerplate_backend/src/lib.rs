#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::fmt::write;
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// struct for property
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct Property {
    id: u64,
    address: String,
    price: f64,
    description: String,
    owner_public_key: Vec<u8>, // Public key of the current owner
}
// Implement storable and boundedstorable traits of property
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

// Property payload
#[derive(candid::CandidType, Serialize, Deserialize)]
struct PropertyPayload {
    address: String,
    price: f64,
    description: String,
    owner_public_key: Vec<u8>, // Public key of the current owner
}

// Function to verify the owner's identity
fn verify_owner_identity(_payload: &PropertyPayload) -> bool {
    // For demo all public keys are valid
    true
}

// Function to get existing properties
#[ic_cdk::query]
fn get_properties() -> Result<Vec<Property>, Error> {
    let properties = PROPERTY_STORAGE.with(|m| {
        m.borrow()
            .iter()
            .map(|(_, v)| v.clone())
            .collect::<Vec<_>>()
    });
    if properties.len() == 0 {
        return Err(Error::NotFound {
            msg: "No properties found".to_string(),
        });
    }
    Ok(properties)
}

//Function to get property by ID
#[ic_cdk::query]
fn get_property_by_id(id: u64) -> Result<Property, Error> {
    PROPERTY_STORAGE.with(|service| {
        service.borrow_mut().get(&id).ok_or(Error::NotFound {
            msg: format!("Property with ID = {} not found", id),
        })
    })
}

//Function to add new property
#[ic_cdk::update]
fn add_property(payload: PropertyPayload) -> Result<Property, Error> {
    //Simple validation to ensure no fields are left empty
    if payload.owner_public_key.is_empty()
        || payload.address.is_empty()
        || payload.description.is_empty()
        || payload.price.is_nan()
    {
        return Err(Error::Validate {
            msg: "Please fill in all the required fields".to_string(),
        });
    }

    // Verify owner before adding new property
    if !verify_owner_identity(&payload) {
        return Err(Error::Unauthorized {
            msg: "Invalid owner identity".to_string(),
        });
    }

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
        owner_public_key: payload.owner_public_key,
    };

    PROPERTY_STORAGE.with(|m| m.borrow_mut().insert(id, property.clone()));
    Ok(property)
}

//Function to update any property info
#[ic_cdk::update]
fn update_property(id: u64, payload: PropertyPayload) -> Result<Property, Error> {
    //Simple validation to ensure no fields are left
    if payload.address.is_empty() & payload.price.is_nan() & payload.description.is_empty() {
        return Err(Error::Validate {
            msg: "You cannot leave all leave all fields empty".to_string(),
        });
    }

    PROPERTY_STORAGE.with(|m| {
        let mut property = m.borrow_mut().get(&id).ok_or(Error::NotFound {
            msg: format!("Property with id = {} not found ", id),
        })?;

        // Verify owner before updating property info
        if !verify_owner_identity(&payload) {
            return Err(Error::Unauthorized {
                msg: "Invalid owner identity".to_string(),
            });
        }

        property.address = payload.address;
        property.price = payload.price;
        property.description = payload.description;

        m.borrow_mut().insert(id, property.clone());
        Ok(property)
    })
}

// Function to delete the property from website when bought
#[ic_cdk::update]
fn delete_property(id: u64) -> Result<Property, Error> {
    PROPERTY_STORAGE.with(|m| {
        m.borrow_mut().remove(&id).ok_or(Error::NotFound {
            msg: format!("Property with id = {} not found", id),
        })
    })
}

// Function to transfer ownership after transactions
#[ic_cdk::update]
fn transfer_property_ownership(id: u64, new_owner: Vec<u8>) -> Result<Property, Error> {
    PROPERTY_STORAGE.with(|m| {
        let mut property = m.borrow_mut().get(&id).ok_or(Error::NotFound{ 
            msg: format!("Property with id = {} not found", id)
        })?;

        property.owner_public_key = new_owner;

        m.borrow_mut().insert(id, property.clone());
        Ok(property)
    })
}

#[derive(candid::CandidType, Deserialize, Serialize, Debug)]
enum Error {
    Validate { msg: String },
    NotFound { msg: String },
    Unauthorized { msg: String },
}

//Provides candid interface.
ic_cdk::export_candid!();