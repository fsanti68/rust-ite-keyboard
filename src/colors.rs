use std::collections::HashMap;

lazy_static! {
   static ref COLORS: HashMap<&'static str, u32> = {
       let mut map = HashMap::new();
       map.insert("black", 0x0);
       map.insert("white", 0xffffff);
       map.insert("gray", 0x808080);
       map.insert("blue", 0x8080ff);
       map
   };
}

pub fn get_color(color_name: &str) -> u32{
   let name = color_name.to_lowercase().to_owned();
   let color: Option<&u32> = COLORS.get(&name[..]);
   match color {
      Some(c) => c.to_owned(),
      _ => 0x0,
   }
}