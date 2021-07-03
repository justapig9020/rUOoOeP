use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

/// Decoder is used to decode instruction and find
/// appropriate name of reservation station
pub struct Decoder {
    stations: HashMap<String, StationList>,
    formats: HashMap<String, InstFormat>,
}

#[cfg(test)]
mod decoder {
    use super::*;
    #[test]
    fn register_mul() {
        use SyntaxType::*;
        let mut d = Decoder::new();
        let test_inst = vec![
        InstFormat {
            name: String::from("add"),
            syntax: vec![Register, Register, Register],
            writeback: true,
        },
        InstFormat {
            name: String::from("addi"),
            syntax: vec![Register, Register, Immediate],
            writeback: false,
        },
        ];
        let station0 = String::from("station0");
        let station1 = String::from("station1");

        d.register(test_inst.clone(), station0.clone()).unwrap();
        d.register(test_inst.clone(), station1.clone()).unwrap();

        for expect_format in test_inst.iter() {
            let name = &expect_format.name;
            let content = d.stations.get(name).unwrap();
            let stations = content.station.borrow();
            assert_eq!(stations[0], station0);
            assert_eq!(stations[1], station1);

            let format = d.formats.get(name).unwrap();
            assert_eq!(expect_format, format);
        }
    }
    #[test]
    fn decode() {
        use SyntaxType::*;
        let mut d = Decoder::new();
        let inst = vec![InstFormat {
            name: String::from("add"),
            syntax: vec![Register, Register, Immediate],
            writeback: true,
        }];
        let station = String::from("station");
        let to_decode = String::from("add R0, R13, #100");

        d.register(inst, station.clone()).unwrap();

        let got = d.decode(&to_decode).unwrap();
        assert_eq!(1, got.stations.len());
        assert_eq!(station, got.stations[0]);

        assert_eq!(3, got.args.len());
        assert_eq!(ArgType::Reg(0), got.args[0]);
        assert_eq!(ArgType::Reg(13), got.args[1]);
        assert_eq!(ArgType::Imm(100), got.args[2]);
    }
    #[test]
    fn invalid_instruction() {
        use SyntaxType::*;
        let mut d = Decoder::new();
        let inst = vec![InstFormat {
            name: String::from("add"),
            syntax: vec![Register, Register, Immediate],
            writeback: true,
        }];
        let station = String::from("station");
        let args = vec![String::from("r0"), String::from("R13"), String::from("#100")];
        let to_decode = String::from("add R0, R13, 100");
        d.register(inst, station.clone()).unwrap();

        assert!(d.decode(&to_decode).is_err());
    }
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            stations: HashMap::new(),
            formats: HashMap::new(),
        }
    }
    pub fn register(&mut self, inst_list: Vec<InstFormat>, station: String) -> Result<(), String> {
        if inst_list.len() == 0 {
            return Ok(());
        }
        if let Some(station_list) = self.stations.get_mut(&inst_list[0].name) {
            station_list.push(&station);
        } else {
            let station = StationList::new(&station);
            for format in inst_list.iter() {
                let inst = &format.name;
                let prev = self.stations.insert(inst.clone(), station.clone());
                if let Some(_) = prev {
                    let msg = format!("Instruction {} has been used by other function unit", inst);
                    return Err(msg);
                }
                self.formats.insert(inst.clone(), format.clone());
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
        if let Some(list) = self.stations.get(inst_name) {
            if let Some(format) = self.formats.get(inst_name) {
                let stations = list.station.borrow();
                let stations = (*stations).clone();
                let mut args = Vec::with_capacity(tokens.len() - 1);
                let syntax = &format.syntax;
                for (token, expect_type) in tokens[1..].iter().zip(syntax.iter()) {
                    let arg = arg_scan(token)?;
                    let get_type = SyntaxType::from(arg);
                    if *expect_type != get_type {
                        let msg = format!("Expect type {:?}, but get type {:?}", *expect_type, get_type);
                        return Err(msg);
                    }
                    args.push(arg);
                }
                return Ok(DecodedInst {
                    name: inst_name.to_string(),
                    stations,
                    args,
                    writeback: format.writeback,
                });
            }
        }
        let msg = format!("Instruct {} has not implemented", inst);
        Err(msg)
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

#[derive(Clone)]
struct StationList {
    station: Rc<RefCell<Vec<String>>>,
}

impl StationList {
    fn new(name: &String) -> Self {
        Self {
            station: Rc::new(
                         RefCell::new(
                             vec![name.clone()]
                             )),
        }
    }
    fn push(&mut self, name: &String) {
        self
            .station
            .borrow_mut()
            .push(name.clone());
    }
}

pub struct DecodedInst {
    name: String,
    stations: Vec<String>,
    args: Vec<ArgType>,
    writeback: bool,
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
    pub fn is_writeback(&self) -> bool {
        self.writeback
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum ArgType {
    Reg(usize),
    Imm(u32),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SyntaxType {
    Register,
    Immediate,
}

impl SyntaxType {
    fn from(arg: ArgType) -> Self {
        match arg {
            ArgType::Reg(_) => SyntaxType::Register,
            ArgType::Imm(_) => SyntaxType::Immediate,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstFormat {
    name: String,
    syntax: Vec<SyntaxType>,
    writeback: bool,
}
