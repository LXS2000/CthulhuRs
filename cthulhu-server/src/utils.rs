use std::{
    fs,
    io::{self, Read, Write},
    ops::Add,
    path::Path,
};

pub fn mini_match(pattern: &str, target: &str) -> bool {
    let mut pi = 0;
    let mut ti = 0;
    let mut pattern_chars = pattern.chars();
    let mut target_chars = target.chars();
    loop {
        pi += 1;
        ti += 1;
        match (pattern_chars.next(), target_chars.next()) {
            (Some('?'), Some(_)) => continue,
            (Some('?'), None) => return false,
            (Some('*'), None) => return true,
            (Some('*'), Some(t)) => {
                //假设*匹配0次
                let pat = &pattern[pi..];
                let tar = &target[ti - 1..];
                if mini_match(pat, tar) {
                    return true;
                }
                //终止字符
                let tag = match pattern_chars.next() {
                    Some(p) => p,
                    None => return true,
                };
                //遇见终止符
                if tag == t {
                    //假设后续还有终止字符，跳过此次终止字符
                    let pat = &pattern[pi - 1..];
                    let tar = &target[ti..];
                    if mini_match(pat, tar) {
                        return true;
                    }
                    //直接终止
                    continue;
                }
                for t in target_chars {
                    ti += 1;
                    if tag == t {
                        //假设后续还有终止字符，跳过此次终止字符
                        let pat = &pattern[pi..];
                        let tar = &target[ti - 1..];
                        if mini_match(pat, tar) {
                            return true;
                        }
                        //直接终止
                        break;
                    }
                }
                return false;
            }
            (Some(p), Some(t)) => {
                if p == '\\' {
                    let pat = &pattern[pi..pi + 1];
                    if pat == "\\" || pat == "*" || pat == "?" {
                        if t.to_string() != pat {
                            return false;
                        }
                        let pat = &pattern[pi + 1..];
                        let tar = &target[ti..];
                        return mini_match(pat, tar);
                    }
                }
                if p != t {
                    return false;
                }
            }
            (None, None) => return true,
            _ => return false,
        }
    }
}
pub fn hash<'a>(bytes: &'a [u8], size: u8, codes: u8) -> String {
    let mut buf = vec![0; size as usize];

    let codes = match codes {
        1 => "0123456789",
        2 => "abcdefghijklmnopqrstuvwxyz",
        3 => "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        12 => "0123456789abcdefghijklmnopqrstuvwxyz",
        13 => "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        23 => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
        123 => "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
        _ => {
            panic!("invalid codes");
        }
    }.as_bytes();
    let codes_len=codes.len() as u8;
    let fl: usize = buf.len();
    let bl: usize = bytes.len();
    let len = usize::max(fl, bl).add(size as usize);
    let mut seed: usize = size as usize;
    for i in 0..len {
        let bi = i % bl;
        let fi = i % fl;
        seed = seed.wrapping_add(bytes[bi] as usize);
        let bi = seed % bl;
        let bb = bytes[bi];
        let bj = seed.wrapping_add(bb as usize) as usize % bl;
        let bj = bytes[bj];
        *(&mut buf[fi]) = bb.wrapping_add(bj);
    }
    let mut r: u8 = size;
    for i in 0..fl {
        let b = buf[i];
        let j = (r.wrapping_add(b)) % codes_len;
        r = r.wrapping_add(r.wrapping_sub(j));
        buf[i] = codes[j as usize];
    }
    String::from_utf8(buf).unwrap_or_default()
}

pub fn read_bytes<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    match std::fs::File::open(path) {
        Ok(mut f) => {
            let mut vec = vec![];
            f.read_to_end(&mut vec).unwrap();
            Ok(vec)
        }
        Err(e) => Err(e),
    }
}

pub fn write_bytes<P: AsRef<Path>>(path: P, buf: &[u8], is_append: Option<bool>) -> io::Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        if let Some(p) = path.parent() {
            fs::create_dir_all(p)?;
        }
    } else if path.is_file() {
        if let Some(p) = path.parent() {
            fs::create_dir_all(p)?;
        }
    }
    let is_append = is_append.unwrap_or(false);

    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(is_append)
        .write(true)
        .open(path)?;

    f.write_all(&buf).unwrap();
    Ok(())
}
pub fn remove_path<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    }
}
#[allow(dead_code)]
pub fn exists_path<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    path.exists()
}
#[test]
fn a() {
    // assert_eq!(matches_pattern("a.cff.555.*", "a.cff.555.com"), true);
    // assert_eq!(mini_match("a*g", "aagg"), true);
    // println!("{}",mini_match("*.cff.555.co?", "a.cff.555.com")== true);
    // println!("{}", mini_match("a*aa", "aba") == true);
    println!("{}", mini_match("a\\*a", "a*a") == true);
    println!("{}", mini_match("*a.*g", "a.g") == true);
    println!("{}", mini_match("*a*.*g*", "a.g") == true);
    println!("{}", mini_match("*a**.*g*", "a.g") == true);
    println!("{}", mini_match("a.*.*com", "a.cff.555.com") == true);

    // println!("{}", hash("bytes".as_bytes(), 5));
    // println!("{}", hash("bytes".as_bytes(), 255));
}
// #[test]
// fn b() {
//     let mut map = HashMap::new();
//     let mut times = 0;

//     let mut random: StdRng = rand::SeedableRng::seed_from_u64(0);

//     for item in 0..1000000 {
//         let a = random.next_u32();
//         let b = random.next_u32();
//         let value = format!("{a}.{b}");
//         let bytes = value.as_bytes();
//         let hash = hash(bytes, 16);
//         let set = match map.get_mut(&hash) {
//             Some(v) => v,
//             None => {
//                 map.insert(hash.clone(), HashSet::new());
//                 map.get_mut(&hash).unwrap()
//             }
//         };
//         if !set.is_empty() && !set.contains(&value) {
//             times += 1;
//         } else {
//             set.insert(value);
//         }
//         if item % 1000 == 0 {
//             println!("i={item} {hash}");
//         }
//     }
//     println!("times={times}");
// }
