use quick_xml::{Reader, events::Event};
use std::io::Cursor;

fn main() {
    let data = r#"<?xml version="1.0"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
  <library_geometries>
    <geometry id="simple-triangle">
      <mesh>
        <source id="positions">
          <float_array id="positions-array" count="9">
            0.0 0.0 0.0 1.0 0.0 0.0 0.0 1.0 0.0
          </float_array>
        </source>
        <vertices id="vertices">
          <input semantic="POSITION" source="#positions"/>
        </vertices>
        <triangles count="1">
          <input semantic="VERTEX" source="#vertices" offset="0"/>
          <p>0 1 2</p>
        </triangles>
      </mesh>
    </geometry>
  </library_geometries>
</COLLADA>"#;

    let mut reader = Reader::from_reader(Cursor::new(data.as_bytes()));
    reader.trim_text(true);
    
    let mut buf = Vec::new();
    let mut depth = 0;
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref());
                let attrs = e.attributes()
                    .map(|a| {
                        let attr = a.unwrap();
                        format!("{}={}", 
                            String::from_utf8_lossy(attr.key.as_ref()),
                            String::from_utf8_lossy(&attr.value))
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                
                println!("{:indent$}<{} {}>", "", name, attrs, indent = depth * 2);
                depth += 1;
            }
            Ok(Event::End(ref e)) => {
                depth -= 1;
                let name = String::from_utf8_lossy(e.name().as_ref());
                println!("{:indent$}</{}>", "", name, indent = depth * 2);
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default();
                if !text.trim().is_empty() {
                    println!("{:indent$}[{}]", "", text.trim(), indent = depth * 2);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                println!("Error at position {}: {:?}", reader.buffer_position(), e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }
}
