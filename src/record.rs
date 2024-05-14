use std::{ffi::IntoStringError, marker::PhantomData, net::IpAddr, ptr::null_mut, time::Duration};

use libbgpstream_sys::{
    bgpstream_get_next_record,
    bgpstream_record_status_t_BGPSTREAM_RECORD_STATUS_CORRUPTED_RECORD as RECORD_STATUS_CORRUPTED_RECORD,
    bgpstream_record_status_t_BGPSTREAM_RECORD_STATUS_CORRUPTED_SOURCE as RECORD_STATUS_CORRUPTED_SOURCE,
    bgpstream_record_status_t_BGPSTREAM_RECORD_STATUS_EMPTY_SOURCE as RECORD_STATUS_EMPTY_SOURCE,
    bgpstream_record_status_t_BGPSTREAM_RECORD_STATUS_FILTERED_SOURCE as RECORD_STATUS_FILTERED_SOURCE,
    bgpstream_record_status_t_BGPSTREAM_RECORD_STATUS_OUTSIDE_TIME_INTERVAL as RECORD_STATUS_OUTSIDE_TIME_INTERVAL,
    bgpstream_record_status_t_BGPSTREAM_RECORD_STATUS_UNSUPPORTED_RECORD as RECORD_STATUS_UNSUPPORTED_RECORD,
    bgpstream_record_status_t_BGPSTREAM_RECORD_STATUS_VALID_RECORD as RECORD_STATUS_VALID_RECORD,
    bgpstream_record_t, bgpstream_record_type_t_BGPSTREAM_RIB as BGPSTREAM_RIB,
    bgpstream_record_type_t_BGPSTREAM_UPDATE as BGPSTREAM_UPDATE,
};
use time::OffsetDateTime;

use crate::{
    array_to_string, element::Element, parse_bgpstream_ip, stream::BgpStream, BgpStreamError,
};

pub struct Record<'a> {
    pub(crate) p_record: *mut bgpstream_record_t,
    pub(crate) time: OffsetDateTime,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Record<'a> {
    pub(crate) fn new(record: &'a mut BgpStream) -> Result<Option<Record<'a>>, BgpStreamError> {
        unsafe {
            let mut p_record = null_mut::<bgpstream_record_t>();
            let p_p_record: *mut *mut bgpstream_record_t = &mut p_record;
            let res = bgpstream_get_next_record(record.bs.as_ptr(), p_p_record);
            if res == 0 {
                return Ok(None);
            } else if res.is_negative() {
                return Err(BgpStreamError::GetNextRecord);
            }

            // check that p_record is non-null
            if p_record.is_null() {
                return Err(BgpStreamError::GetNextRecordNull);
            };
            // get a reference to the record
            let record = &*p_record;

            // check the record
            match record.status {
                RECORD_STATUS_VALID_RECORD => {}
                RECORD_STATUS_FILTERED_SOURCE
                | RECORD_STATUS_EMPTY_SOURCE
                | RECORD_STATUS_CORRUPTED_SOURCE => {
                    return Err(BgpStreamError::RecordSourceEmptyOrCorrupted)
                }
                RECORD_STATUS_OUTSIDE_TIME_INTERVAL => {
                    return Ok(None)
                }
                RECORD_STATUS_CORRUPTED_RECORD => return Err(BgpStreamError::RecordCorrupted),
                RECORD_STATUS_UNSUPPORTED_RECORD => return Err(BgpStreamError::RecordUnsupported),
                _ => return Err(BgpStreamError::UnknownRecordStatus),
            }

            // compute the time
            let secs = record.time_sec;
            let micros = record.time_usec;

            let time = OffsetDateTime::from_unix_timestamp(secs as i64)?
                + Duration::from_micros(micros as u64);

            Ok(Some(Record {
                p_record,
                _phantom: PhantomData,
                time,
            }))
        }
    }

    pub fn is_update(&self) -> Result<bool, BgpStreamError> {
        unsafe {
            let record = &*self.p_record;
            match record.type_ {
                BGPSTREAM_UPDATE => Ok(true),
                BGPSTREAM_RIB => Ok(false),
                _ => Err(BgpStreamError::RecordCorrupted),
            }
        }
    }

    pub fn time(&self) -> OffsetDateTime {
        self.time
    }

    pub fn project_name(&self) -> Result<String, IntoStringError> {
        unsafe {
            let record = &*self.p_record;
            // get the bytes for the raw name
            array_to_string(&record.project_name)
        }
    }

    pub fn collector_name(&self) -> Result<String, IntoStringError> {
        unsafe {
            let record = &*self.p_record;
            // get the bytes for the raw name
            array_to_string(&record.collector_name)
        }
    }

    pub fn router_name(&self) -> Result<String, IntoStringError> {
        unsafe {
            let record = &*self.p_record;
            // get the bytes for the raw name
            array_to_string(&record.router_name)
        }
    }

    pub fn router_ip(&self) -> Result<IpAddr, BgpStreamError> {
        unsafe {
            let record = &*self.p_record;
            parse_bgpstream_ip(record.router_ip)
        }
    }

    /// Get the next element and return it.
    pub fn next_element(&mut self) -> Result<Option<Element>, BgpStreamError> {
        Element::new(self)
    }

    /// Detach `self` to get a static Record.
    ///
    /// **Safety**: Ensure that there only ever exists a single record for any `BgpStream`.
    pub(crate) unsafe fn detach(self) -> Record<'static> {
        Record {
            p_record: self.p_record,
            time: self.time,
            _phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for Record<'a> {
    type Item = Result<Element, BgpStreamError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_element().transpose()
    }
}
