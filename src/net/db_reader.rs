use super::Asn;
use log::{debug, warn};
use maxminddb::geoip2;
use std::net::IpAddr;

static AS_ISP_DB_PATH: &str = "./src/net/geolite2/GeoLite2-ASN_20260112/GeoLite2-ASN.mmdb";

pub struct DbReader {
    reader: maxminddb::Reader<Vec<u8>>,
}

impl DbReader {
    pub fn new() -> Self {
        let reader =
            maxminddb::Reader::open_readfile(AS_ISP_DB_PATH).expect("Error opening database");
        debug!("Succesfully opened AS database.");
        DbReader { reader }
    }

    pub fn lookup_asn(&self, ip: IpAddr) -> Option<Asn> {
        let mut found_asn = None;
        if let Ok(result) = self.reader.lookup(ip)
            && let Ok(result) = result.decode::<geoip2::Asn>()
        {
            if let Some(asn) = result {
                found_asn = asn.autonomous_system_number;
            } else {
                warn!("ASN for {ip} not available");
            }
        }
        found_asn
    }
}

impl Default for DbReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn valid_ip_asn_lookup() {
        let db_reader = DbReader::new();
        let example: IpAddr = FromStr::from_str("46.229.165.147").unwrap();
        let actual = db_reader.lookup_asn(example);
        let expected = Some(39572);
        assert_eq!(actual, expected);
    }

    #[test]
    fn invalid_ip_asn_lookup() {
        let db_reader = DbReader::new();
        let zero_addr: IpAddr = FromStr::from_str("0.0.0.0").unwrap();
        let actual = db_reader.lookup_asn(zero_addr);
        assert!(actual.is_none());
    }

    #[test]
    fn valid_ipv6_lookup() {
        let db_reader = DbReader::new();
        let google: IpAddr = FromStr::from_str("2a00:1450:4005:80b::200e").unwrap();
        let actual = db_reader.lookup_asn(google);
        let expected = Some(15169);
        assert_eq!(actual, expected);
    }
}
