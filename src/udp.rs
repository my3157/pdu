/*
   Copyright (c) 2019 Alex Forster <alex@alexforster.com>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.

   SPDX-License-Identifier: Apache-2.0
*/

use core::convert::TryInto;

use crate::{util, Error, Result};

/// Represents a UDP header and payload
#[derive(Debug, Copy, Clone)]
pub struct UdpPdu<'a> {
    buffer: &'a [u8],
}

/// Contains the inner payload of a [`UdpPdu`]
#[derive(Debug, Copy, Clone)]
pub enum Udp<'a> {
    Raw(&'a [u8]),
}

impl<'a> UdpPdu<'a> {
    /// Constructs a [`UdpPdu`] backed by the provided `buffer`
    #[cfg_attr(feature = "inline", inline(always))]
    pub fn new(buffer: &'a [u8]) -> Result<Self> {
        let pdu = UdpPdu { buffer };
        if buffer.len() < 8 || (pdu.length() as usize) < buffer.len() {
            return Err(Error::Truncated);
        }
        Ok(pdu)
    }

    /// Returns a reference to the entire underlying buffer that was provided during construction
    #[cfg_attr(feature = "inline", inline(always))]
    pub fn buffer(&'a self) -> &'a [u8] {
        self.buffer
    }

    /// Returns the slice of the underlying buffer that contains the header part of this PDU
    #[cfg_attr(feature = "inline", inline(always))]
    pub fn as_bytes(&'a self) -> &'a [u8] {
        &self.buffer[0..8]
    }

    /// Returns an object representing the inner payload of this PDU
    #[cfg_attr(feature = "inline", inline(always))]
    pub fn inner(&'a self) -> Result<Udp<'a>> {
        Ok(Udp::Raw(&self.buffer[8..]))
    }

    #[cfg_attr(feature = "inline", inline(always))]
    pub fn source_port(&'a self) -> u16 {
        u16::from_be_bytes(self.buffer[0..=1].try_into().unwrap())
    }

    #[cfg_attr(feature = "inline", inline(always))]
    pub fn destination_port(&'a self) -> u16 {
        u16::from_be_bytes(self.buffer[2..=3].try_into().unwrap())
    }

    #[cfg_attr(feature = "inline", inline(always))]
    pub fn length(&'a self) -> u16 {
        u16::from_be_bytes(self.buffer[4..=5].try_into().unwrap())
    }

    #[cfg_attr(feature = "inline", inline(always))]
    pub fn checksum(&'a self) -> u16 {
        u16::from_be_bytes(self.buffer[6..=7].try_into().unwrap())
    }

    #[cfg_attr(feature = "inline", inline(always))]
    pub fn computed_checksum(&'a self, ip: &crate::Ip) -> u16 {
        match ip {
            crate::Ip::Ipv4(ipv4) => util::checksum(&[
                &ipv4.source_address().as_ref(),
                &ipv4.destination_address().as_ref(),
                &[0x00, ipv4.protocol()].as_ref(),
                &self.length().to_be_bytes().as_ref(),
                &self.buffer[0..=5],
                &self.buffer[8..],
            ]),
            crate::Ip::Ipv6(ipv6) => util::checksum(&[
                &ipv6.source_address().as_ref(),
                &ipv6.destination_address().as_ref(),
                &(self.length() as u32).to_be_bytes().as_ref(),
                &[0x0, 0x0, 0x0, ipv6.computed_protocol()].as_ref(),
                &self.buffer[0..=5],
                &self.buffer[8..],
            ]),
        }
    }
}
