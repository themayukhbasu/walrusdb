mod best_fit;

use crate::kvdb::errors::DBError;
use crate::kvdb::pager::ByteRange;
use crate::kvdb::pager::page::{PAGE_SIZE, Page};
use std::collections::HashMap;

const FREE_LIST_REGION_SIZE: usize = PAGE_SIZE / 4;
const MAX_PAGES: usize = FREE_LIST_REGION_SIZE / size_of::<u16>();

#[derive(Debug, PartialEq)]
struct FreeListPage {
    // free list
    // array ->  [ total free space ] | index is the page number
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

#[derive(Debug, PartialEq)]
struct AvailabilityList {
    // availability list
    // a map of map -> { page_num :  [cell ptr] }
    availability_list: HashMap<u16, Vec<ByteRange>>,
}

pub struct FreeListManager {
    free_list_page: FreeListPage,
}

impl FreeListManager {
    pub fn load(free_list_page_bytes: [u8; PAGE_SIZE]) -> Result<Self, DBError> {
        let free_list_page = FreeListPage::decode(free_list_page_bytes)?;

        Ok(Self { free_list_page })
    }

    pub fn flush(&self) -> Result<[u8; PAGE_SIZE], DBError> {
        let buf = self.free_list_page.encode()?;
        Ok(buf)
    }
}

impl FreeListPage {
    fn new() -> Self {
        let new_free_list = FreeList {
            free_list: [0u16; MAX_PAGES],
        };
        let new_availability_list = AvailabilityList {
            availability_list: HashMap::new(),
        };
        Self {
            free_list: new_free_list,
            availability_list: new_availability_list,
        }
    }

    fn encode(&self) -> Result<[u8; PAGE_SIZE], DBError> {
        let free_list_bytes = self.free_list.encode()?;
        let availability_list_bytes = self.availability_list.encode()?;
        let mut buf = [0u8; PAGE_SIZE];
        buf[0..FREE_LIST_REGION_SIZE].copy_from_slice(&free_list_bytes);
        buf[FREE_LIST_REGION_SIZE..PAGE_SIZE].copy_from_slice(&availability_list_bytes);
        Ok(buf)
    }

    fn decode(buf: [u8; PAGE_SIZE]) -> Result<Self, DBError> {
        let free_list = FreeList::decode(
            buf[0..FREE_LIST_REGION_SIZE]
                .try_into()
                .expect("fixed size"),
        )?;
        let availability_list = AvailabilityList::decode(
            buf[FREE_LIST_REGION_SIZE..PAGE_SIZE]
                .try_into()
                .expect("fixed size"),
        );

        Ok(Self {
            free_list,
            availability_list,
        })
    }
}

impl FreeList {
    fn encode(&self) -> Result<[u8; FREE_LIST_REGION_SIZE], DBError> {
        let mut buf = [0u8; FREE_LIST_REGION_SIZE];
        for page_num in 1..MAX_PAGES {
            // index 0 is for page 0 which is free list page
            if self.free_list[page_num] > PAGE_SIZE as u16 {
                return Err(DBError::InvalidFreeSpace(
                    self.free_list[page_num],
                    String::from("free space cannot be greater than page size"),
                ));
            }
            let start_idx = page_num * size_of::<u16>();
            buf[start_idx..start_idx + size_of::<u16>()]
                .copy_from_slice(&self.free_list[page_num].to_le_bytes());
        }
        Ok(buf)
    }

    fn decode(buf: [u8; FREE_LIST_REGION_SIZE]) -> Result<Self, DBError> {
        let mut free_list: [u16; MAX_PAGES] = [0u16; MAX_PAGES];
        for page_num in 1..MAX_PAGES {
            // index 0 is for page 0 which is free list page
            let start_idx = page_num * size_of::<u16>();
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
            free_list[page_num] = free_space;
        }
        Ok(Self { free_list })
    }
}

impl AvailabilityList {
    fn encode(&self) -> Result<[u8; PAGE_SIZE - FREE_LIST_REGION_SIZE], DBError> {
        let mut buf: Vec<u8> = Vec::new();
        for (page_num, free_slots) in &self.availability_list {
            for free_slot in free_slots {
                let page_num_bytes: [u8; 2] = page_num.to_le_bytes();
                let cell_ptr_bytes: [u8; 4] = free_slot.encode();
                let free_slot_bytes: Vec<u8> =
                    [page_num_bytes.as_slice(), cell_ptr_bytes.as_slice()].concat();
                buf.extend(free_slot_bytes);
            }
        }
        if buf.len() > (PAGE_SIZE - FREE_LIST_REGION_SIZE) {
            return Err(DBError::AvailabilityListOverflow(
                buf.len(),
                PAGE_SIZE - FREE_LIST_REGION_SIZE,
                String::from("Try Compaction"),
            ));
        }
        buf.resize(PAGE_SIZE - FREE_LIST_REGION_SIZE, 0u8);
        let buf = buf.as_slice().try_into().expect("fixed size");
        Ok(buf)
    }

    fn decode(buf: [u8; PAGE_SIZE - FREE_LIST_REGION_SIZE]) -> Self {
        let mut availability_list: HashMap<u16, Vec<ByteRange>> = HashMap::new();
        for slot in buf.chunks_exact(6) {
            let page_num = u16::from_le_bytes(slot[0..2].try_into().expect("Fixed size"));
            if page_num == 0 {
                // hit padding
                break;
            }
            let cell_ptr = ByteRange::decode(slot[2..6].try_into().expect("Fixed size"));
            availability_list
                .entry(page_num)
                .and_modify(|slots| slots.push(cell_ptr))
                .or_insert(vec![cell_ptr]);
        }
        Self { availability_list }
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
        let mut full_space_free_list = FreeList {
            free_list: {
                let mut fl = [PAGE_SIZE as u16; MAX_PAGES];
                fl[0] = 0;
                fl
            },
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
        buf[size_of::<u16>()..2 * size_of::<u16>()].copy_from_slice(&u16::MAX.to_le_bytes());
        assert!(FreeList::decode(buf).is_err());
    }

    #[test]
    fn availability_list_round_trip_matches() {
        let empty_al = AvailabilityList {
            availability_list: HashMap::new(),
        };

        assert_eq!(
            empty_al,
            AvailabilityList::decode(empty_al.encode().unwrap())
        )
    }

    #[test]
    fn availability_list_overflow_raises_error() {
        let mut availability_list = HashMap::new();
        availability_list.insert(
            10,
            vec![
                ByteRange {
                    offset: 10,
                    len: 10
                };
                ((PAGE_SIZE - FREE_LIST_REGION_SIZE) / 6) + 1   // size greater than max possible slots
            ],
        );
        let overflow_al = AvailabilityList { availability_list };

        assert!(overflow_al.encode().is_err());
        assert!(matches!(
            overflow_al.encode(),
            Err(DBError::AvailabilityListOverflow(_, _, _))
        ));
    }

    #[test]
    fn free_list_page_round_trip_matches() {
        let free_list = FreeList {
            free_list: {
                let mut fl = [0u16; MAX_PAGES];
                fl[1] = 100;
                fl[2] = 4096;
                fl[MAX_PAGES - 1] = 50; // last slot - checks the far edge of the region too
                fl
            },
        };

        let mut availability_map = HashMap::new();
        availability_map.insert(
            1,
            vec![ByteRange {
                offset: 20,
                len: 80,
            }],
        );
        availability_map.insert(
            2,
            vec![
                ByteRange {
                    offset: 100,
                    len: 200,
                },
                ByteRange {
                    offset: 4000,
                    len: 96,
                },
            ],
        );
        let availability_list = AvailabilityList {
            availability_list: availability_map,
        };
        let page = FreeListPage {
            free_list,
            availability_list,
        };

        assert_eq!(page, FreeListPage::decode(page.encode().unwrap()).unwrap());
    }
}
