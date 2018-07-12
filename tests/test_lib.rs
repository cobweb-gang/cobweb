extern crate cobweb;

mod test {
    extern crate mac_address;

    use cobweb::node::get_info;
    use cobweb::node::Info;
    use self::mac_address::get_mac_address;

    #[test]
    fn mac_addr() {
        let info = get_info();
        let test_info = Info {
            mac: get_mac_address().unwrap(),
            source: false,
            web_connected: false,
        };

        assert_eq!(
            test_info.mac,
            info.mac
        );
    }
}
