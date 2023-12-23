#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Debt {
    id: u64,
    debtor: String,
    creditor: String,
    amount: u64,
    created_at: u64,
}

impl Storable for Debt {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Debt {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Escrow {
    debt_id: u64,
    amount: u64,
    created_at: u64,
}

impl Storable for Escrow {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Escrow {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static DEBT_STORAGE: RefCell<StableBTreeMap<u64, Debt, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static ESCROW_STORAGE: RefCell<StableBTreeMap<u64, Escrow, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct DebtPayload {
    debtor: String,
    creditor: String,
    amount: u64,
}

#[ic_cdk::query]
fn get_debt(id: u64) -> Result<Debt, Error> {
    match _get_debt(&id) {
        Some(debt) => Ok(debt),
        None => Err(Error::NotFound {
            msg: format!("a debt with id={} not found", id),
        }),
    }
}

#[ic_cdk::query]
fn get_escrow(debt_id: u64) -> Result<Escrow, Error> {
    match _get_escrow(&debt_id) {
        Some(escrow) => Ok(escrow),
        None => Err(Error::NotFound {
            msg: format!("escrow for debt_id={} not found", debt_id),
        }),
    }
}

#[ic_cdk::update]
fn add_debt(debt: DebtPayload) -> Option<Debt> {
    // Validate input data
    if debt.debtor.is_empty() || debt.creditor.is_empty() || debt.amount == 0 {
        return None; // Invalid input, return early
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let debt = Debt {
        id,
        debtor: debt.debtor,
        creditor: debt.creditor,
        amount: debt.amount,
        created_at: time(),
    };

    do_insert_debt(&debt);
    Some(debt)
}

#[ic_cdk::update]
fn update_debt(id: u64, payload: DebtPayload) -> Result<Debt, Error> {
    // Validate input data
    if payload.debtor.is_empty() || payload.creditor.is_empty() || payload.amount == 0 {
        return Err(Error::InvalidInput {
            msg: "Invalid input data".to_string(),
        });
    }

    match DEBT_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut debt) => {
            debt.debtor = payload.debtor;
            debt.creditor = payload.creditor;
            debt.amount = payload.amount;
            do_insert_debt(&debt);
            Ok(debt)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update a debt with id={}. debt not found",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn create_escrow(debt_id: u64, amount: u64) -> Result<Escrow, Error> {
    // Validate input data
    if amount == 0 {
        return Err(Error::InvalidInput {
            msg: "Invalid escrow amount".to_string(),
        });
    }

    match DEBT_STORAGE.with(|service| service.borrow().get(&debt_id)) {
        Some(_) => {
            let escrow_id = ID_COUNTER
                .with(|counter| {
                    let current_value = *counter.borrow().get();
                    counter.borrow_mut().set(current_value + 1)
                })
                .expect("cannot increment id counter");

            let escrow = Escrow {
                debt_id,
                amount,
                created_at: time(),
            };

            do_insert_escrow(&escrow);
            Ok(escrow)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't create escrow for debt_id={}. debt not found",
                debt_id
            ),
        }),
    }
}

fn do_insert_debt(debt: &Debt) {
    DEBT_STORAGE.with(|service| service.borrow_mut().insert(debt.id, debt.clone()));
}

fn do_insert_escrow(escrow: &Escrow) {
    ESCROW_STORAGE
        .with(|service| service.borrow_mut().insert(escrow.debt_id, escrow.clone()));
}

#[ic_cdk::update]
fn delete_debt(id: u64) -> Result<Debt, Error> {
    match DEBT_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(debt) => Ok(debt),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a debt with id={}. debt not found.",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn release_escrow(debt_id: u64) -> Result<u64, Error> {
    match ESCROW_STORAGE.with(|service| service.borrow_mut().remove(&debt_id)) {
        Some(escrow) => Ok(escrow.amount),
        None => Err(Error::NotFound {
            msg: format!("escrow for debt_id={} not found", debt_id),
        }),
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    InvalidInput { msg: String },
}

fn _get_debt(id: &u64) -> Option<Debt> {
    DEBT_STORAGE.with(|service| service.borrow().get(id))
}

fn _get_escrow(debt_id: &u64) -> Option<Escrow> {
    ESCROW_STORAGE.with(|service| service.borrow().get(debt_id))
}

#[ic_cdk::update]
fn delete_escrow(debt_id: u64) -> Result<Escrow, Error> {
    match ESCROW_STORAGE.with(|service| service.borrow_mut().remove(&debt_id)) {
        Some(escrow) => Ok(escrow),
        None => Err(Error::NotFound {
            msg: format!("escrow for debt_id={} not found", debt_id),
        }),
    }
}

#[ic_cdk::update]
fn update_escrow(debt_id: u64, new_amount: u64) -> Result<Escrow, Error> {
    // Validate input data
    if new_amount == 0 {
        return Err(Error::InvalidInput {
            msg: "Invalid escrow amount".to_string(),
        });
    }

    match ESCROW_STORAGE.with(|service| service.borrow().get(&debt_id)) {
        Some(mut escrow) => {
            escrow.amount = new_amount;
            do_insert_escrow(&escrow);
            Ok(escrow)
        }
        None => Err(Error::NotFound {
            msg: format!("escrow for debt_id={} not found", debt_id),
        }),
    }
}

// need this to generate candid
ic_cdk::export_candid!();
