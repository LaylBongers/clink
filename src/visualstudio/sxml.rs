use std::io::Write;
use xml::writer::{XmlEvent, EventWriter};

pub fn write_simple<W: Write>(w: &mut EventWriter<W>, name: &str, value: &str) {
    w.write(XmlEvent::start_element(name)).unwrap();
    w.write(XmlEvent::characters(value)).unwrap();
    w.write(XmlEvent::end_element()).unwrap();
}
