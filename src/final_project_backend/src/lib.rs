use candid::{CandidType,Decode,Deserialize,Encode};
use ic_stable_structures::memory_manager::{MemoryId,MemoryManager,VirtualMemory};
use ic_stable_structures::{BoundedStorable,DefaultMemoryImpl,StableBTreeMap, Storable};
use std::{borrow::Cow,call::RefCell};

type Memory= VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUE_SIZE: u32 =5000;

#[derive(CandidType)]
enum BidError{
    ListIsNotActive,
    NoSuchItem,
    AccessRejected,
    UpdateError,
}

#[derive(CandidType,Deserialize)]
struct item {
    new_owner: candid:: Principal,
    name:String,
    last_bid: u64,
}

#[derive(CandidType,Deserialize)]
struct Listing{
    description: String,
    items: vec![item],
    is_active: bool,
    owner: candid:: Principal,
}

impl Storable for Listing {

    fn to_bytes (&self) -> Cow<u8> { 
        Cow::Owned(Encode!(self).unwrap()) 
    }

    fn from_bytes (bytes: Cow<[u8]>) -> Self{ 
        Decode!(bytes.as_ref(), Self).unwrap()
    }

}


impl BoundedStorable for Listing{
    const MAX_SİZE :u32 = MAX_VALUE_SIZE;
    const IS_FIXED_SİZE :bool = false;
}


thread_local! {

    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static LISTING_MAP: RefCell<StableBTreeMap<u64,Listing,Memory>> = RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
    ))
}


#[ic_cdk::query]
fn get_listing(key: u64) -> Option<Listing>{
    LISTING_MAP.with(|p| p.borrow().get(&key))
}


#[ic_cdk::query]
fn get_listing_count() -> u64{
    LISTING_MAP.with(|p| p.borrow().len())
}


#[ic_cdk::query]
fn create_listing(key: u64, listing:Listing) -> Option<Listing>{
   let value:Listing = Listing { 
    description:listing.description, 
    items:listing.items, 
    is_active:listing.is_active, 
    owner: ic_cdk::caller(), 
    };
       

    LISTING_MAP.with(|p| p.borrow_mut().insert(key,value))
}




#[ic_cdk::update]
fn edit_listing(key: u64, listing: Listing) -> Result<(), BidError>{
    LISTING_MAP.with(|p|{
        let old_listing_opt: Option<Listing> = p.borrow().get(key);
        let old_listing= match old_listing_opt {
            Some(value) => value,
            None =>return Err(BidError::NoSuchItem)
        };

        if ic_cdk::caller() != old_listing.owner {
            return Err(BidError::AccessRejected);
        }
        let value:Listing= Listing { 
            description: listing.description, 
            items: listing.items, 
            is_active: listing.is_active, 
            owner: ic_cdk::caller() ,
        };

        let res = p.borrow_mut().insert(key,value);

        match res {
            Some(_)=> Ok(()),
            None => Err(BidError::UpdateError)
        }
        
    })
}


#[ic_cdk::update]
fn end_listing(key: u64) -> Result<(), BidError>{
    LISTING_MAP.with(|p|{
        let old_listing: Option<Listing> = p.borrow().get(key);
        let listing= match old_listing {
            Some(value) => value,
            None =>return Err(VoteError::NoSuchProposal)
        };

        if ic_cdk::caller() != old_listing.owner {
            return Err(BidError::AccessRejected);
        }

        listing.is_active=false;

        let res = p.borrow_mut().insert(key,value);

        match res {
            Some(_)=> Ok(()),
            None => Err(BidError::UpdateError)
        }
        
    })
}



#[ic_cdk::update]
fn Bid(key: u64,Item: item, bid :u64) -> Result<(), BidError>{
    LISTING_MAP.with(|p|{
        let old_listing: Option<Listing> = p.borrow().get(key);
        let listing= match old_listing {
            Some(value) => value,
            None =>return Err(VoteError::NoSuchProposal)
        };

        let caller :Principal = ic_cdk::caller();

        if listing.is_active == false{
            return Err(BidError::ListIsNotActive);
        }
        if listing.items.contains(Item) {
            if Item.last_bid < bid{
                Item.new_owner = caller;
            }
        }else {
            return Err(BidError::NoSuchItem);
        }



        let res = p.borrow_mut().insert(key,value);

        match res {
            Some(_)=> Ok(()),
            None => Err(BidError::UpdateError)
        }
        
    })
}


