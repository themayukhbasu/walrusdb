use crate::kvdb::errors::DBError;
use crate::kvdb::pager::page::{PAGE_SIZE, Page};

const FREE_LIST_REGION_SIZE: usize = PAGE_SIZE / 4;
const MAX_PAGES: usize = FREE_LIST_REGION_SIZE / size_of::<u16>();

struct FreeListPage {
    // free list
    // a map ->  { page_num : total free space}
    free_list: FreeList,

    // availability list
    // a map of map -> { page_num : { cell ptr : free space }}
    availability_list: AvailabilityList,
}

#[derive(Debug, PartialEq)]
struct FreeList {
    // free list
    free_list: [u16; MAX_PAGES],
}

struct AvailabilityList {
    // availability list
    // a map of map -> { page_num : { cell ptr : free space }}
}

struct FreeListManager {
    free_list_page: FreeListPage,
}

impl FreeListPage {
    fn new(&self) {}

    fn encode(&self) {}

    fn decode(buf: [u8; PAGE_SIZE]) {}
}

impl FreeList {
    fn encode(&self) -> Result<[u8; FREE_LIST_REGION_SIZE], DBError> {
        let mut buf = [0u8; FREE_LIST_REGION_SIZE];
        for i in 0..MAX_PAGES {
            if self.free_list[i] > PAGE_SIZE as u16 {
                return Err(DBError::InvalidFreeSpace(
                    self.free_list[i],
                    String::from("free space cannot be greater than page size"),
                ));
            }
            let start_idx = i * size_of::<u16>();
            buf[start_idx..start_idx + size_of::<u16>()]
                .copy_from_slice(&self.free_list[i].to_le_bytes());
        }
        Ok(buf)
    }

    fn decode(buf: [u8; FREE_LIST_REGION_SIZE]) -> Result<Self, DBError> {
        let mut free_list: [u16; MAX_PAGES] = [0u16; MAX_PAGES];
        for i in 0..MAX_PAGES {
            let start_idx = i * size_of::<u16>();
            let free_space = u16::from_le_bytes(
                buf[start_idx..start_idx + size_of::<u16>()]
                    .try_into()
                    .expect("Fixed size"),
            );
            if free_space > PAGE_SIZE as u16 {
                return Err(DBError::InvalidFreeSpace(
                    free_space,
                    String::from("free space cannot be greater than page size"),
                ));
            }
            free_list[i] = free_space;
        }
        Ok(Self { free_list })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_list_encode_decode_round_trip_matches() {
        let all_zero_free_list = FreeList {
            free_list: [0u16; MAX_PAGES],
        };
        let full_space_free_list = FreeList {
            free_list: [PAGE_SIZE as u16; MAX_PAGES],
        };

        assert_eq!(
            all_zero_free_list,
            FreeList::decode(all_zero_free_list.encode().unwrap()).unwrap()
        );
        assert_eq!(
            full_space_free_list,
            FreeList::decode(full_space_free_list.encode().unwrap()).unwrap()
        );
    }

    #[test]
    fn free_list_raise_error_for_invalid_free_space() {
        let invalid_space_free_list = FreeList {
            free_list: [u16::MAX; MAX_PAGES],
        };

        assert!(invalid_space_free_list.encode().is_err());

        let mut buf = [0u8; FREE_LIST_REGION_SIZE];
        buf[0..size_of::<u16>()].copy_from_slice(&u16::MAX.to_le_bytes());
        assert!(FreeList::decode(buf).is_err());
    }
}
