use std::collections::HashMap;

lazy_static! {
   static ref COLORS: HashMap<&'static str, (u8, u8, u8)> = {
       let mut map = HashMap::new();
       map.insert("black", (0x0, 0x0, 0x0) );

       map.insert("aqua", (0x0, 0xff, 0xff) );
       map.insert("blue", (0x0, 0x0, 0xff) );
       map.insert("fuchsia", (0xff, 0x0, 0xff) );
       map.insert("gray", (0x80, 0x80, 0x80) );
       map.insert("green", (0x0, 0x80, 0x0) );
       map.insert("lime", (0x0, 0xff, 0x0) );
       map.insert("brown", (0x80, 0x0, 0x0) );
       map.insert("darkblue", (0x0, 0x0, 0x80) );
       map.insert("olive", (0x80, 0x80, 0x0) );
       map.insert("purple", (0x80, 0x0, 0x80) );
       map.insert("red", (0xff, 0x0, 0x0) );
       map.insert("silver", (0xc0, 0xc0, 0xc0) );
       map.insert("teal", (0x0, 0x80, 0x80) );
       map.insert("white", (0xff, 0xff, 0xff) );
       map.insert("yellow", (0xff, 0xff, 0x0) );

       map
   };
}

pub fn get_color_by_name(color_name: &str) -> &'static (u8, u8, u8) {
   let name = color_name.to_lowercase().to_owned();
   let color = COLORS.get(&name[..]);
   match color {
      Some(c) => c,
      _ => COLORS.get("black").unwrap(),
   }
}

pub fn get_color_names() -> Vec<String> {
   let mut v = Vec::new();
   for k in COLORS.keys() {
      v.push(String::from(k.to_owned()));
   }
   v
}