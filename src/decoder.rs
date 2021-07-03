use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

/// Decoder is used to decode instruction and find
/// appropriate name of reservation station
pub struct Decoder {
    mapping: HashMap<&'static str, StationList>,
}

#[cfg(test)]
mod decoder {
    use super::*;
    #[test]
    fn register_mul() {
        let mut d = Decoder::new();
        let test_inst = ["i1", "i2"];
        let station0 = String::from("station0");
        let station1 = String::from("station1");

        d.register(&test_inst, station0.clone()).unwrap();
        d.register(&test_inst, station1.clone()).unwrap();

        for inst in test_inst.iter() {
            let content = d.mapping.get(inst).unwrap();
            let stations = content.station.borrow();
            assert_eq!(stations[0], station0);
            assert_eq!(stations[1], station1);
        }
    }
    #[test]
    fn decode() {
        let mut d = Decoder::new();
        let inst = ["add"];
        let station = String::from("station");
        let args = ["r0", "R13", "#100"];
        let to_decode = format!("{} {} {} {}", inst[0], args[0], args[1], args[2]);
        d.register(&inst, station.clone()).unwrap();

        let got = d.decode(&to_decode).unwrap();
        assert_eq!(1, got.stations.len());
        assert_eq!(station, got.stations[0]);

        assert_eq!(args.len(), got.args.len());
        assert_eq!(ArgType::Reg(0), got.args[0]);
        assert_eq!(ArgType::Reg(13), got.args[1]);
        assert_eq!(ArgType::Imm(100), got.args[2]);
    }
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            mapping: HashMap::new(),
        }
    }
    pub fn register(&mut self, inst_list: &[&'static str], name: String) -> Result<(), String> {
        if inst_list.len() == 0 {
            return Ok(());
        }
        if let Some(list) = self.mapping.get_mut(inst_list[0]) {
            let mut station = list.station.borrow_mut();
            station.push(name);
        } else {
            let station = Rc::new(RefCell::new(vec![name]));
            for inst in inst_list.iter() {
                let list = StationList {
                    station: station.clone(),
                };
                let prev = self.mapping.insert(*inst, list);
                if let Some(_) = prev {
                    let msg = format!("Instruction {} has been used by other function unit", inst);
                    return Err(msg);
                }
            }
        }
        Ok(())
    }
    pub fn decode(&self, inst: &str) -> Result<DecodedInst, String> {
        let tokens = text_slicer(inst);
        if tokens.len() == 0 {
            let msg = format!("No token has been found in instruction {}", inst);
            return Err(msg);
        }
        let inst_name = tokens[0];
        if let Some(list) = self.mapping.get(inst_name) {
            let stations = list.station.borrow();
            let stations = (*stations).clone();
            let mut args = Vec::with_capacity(tokens.len() - 1);
            for token in tokens[1..].iter() {
                let arg = arg_scan(token)?;
                args.push(arg);
            }
            Ok(DecodedInst {
                name: inst_name.to_string(),
                stations,
                args,
            })
        } else {
            let msg = format!("Instruct {} has not implemented", inst);
            Err(msg)
        }
    }
}

fn arg_scan(token: &str) -> Result<ArgType, String> {
    let mut chars = token.chars();
    let prefix = chars.nth(0).unwrap();
    let token = chars.as_str();
    if prefix == 'r' || prefix == 'R' {
        if let Ok(idx) = token.parse() {
            Ok(ArgType::Reg(idx))
        } else {
            let msg = format!("Expect an integer, found {}", token);
            Err(msg)
        }
    } else if prefix == '#' {
        if let Ok(val) = token.parse() {
            Ok(ArgType::Imm(val))
        } else {
            let msg = format!("Expect an integer, found {}", token);
            Err(msg)
        }
    } else {
        let msg = format!("Invalid argument {}", token);
        Err(msg)
    }
}

fn text_slicer<'a>(txt: &'a str) -> Vec<&'a str> {
    let mut begin = 0;
    let mut v = Vec::new();
    let delimiters = [' ', ',', '(', ')', ':', '\n'];
    for (idx, c) in txt.chars().into_iter().enumerate() {
        if delimiters.iter().any(|&d| d == c) {
            if begin != idx {
                v.push(&txt[begin..idx]);
            }
            begin = idx + 1;
        } else if idx == txt.len() - 1 {
            v.push(&txt[begin..=idx]);
        }
    }
    v
}

struct StationList {
    station: Rc<RefCell<Vec<String>>>,
}

pub struct DecodedInst {
    name: String,
    stations: Vec<String>,
    args: Vec<ArgType>,
}

impl DecodedInst {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_stations<'a>(&'a self) -> &'a Vec<String> {
        &self.stations
    }
    pub fn get_args<'a>(&'a self) -> &'a Vec<ArgType> {
        &self.args
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum ArgType {
    Reg(usize),
    Imm(u32),
}
