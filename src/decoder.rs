use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
/// Decoder is used to decode instruction and find
/// appropriate name of reservation station
pub struct Decoder {
    instruction: String,
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
        let _args = vec![String::from("r0"), String::from("R13"), String::from("#100")];
        let to_decode = String::from("add R0, R13, 100");
        d.register(inst, station.clone()).unwrap();

        assert!(d.decode(&to_decode).is_err());
    }
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            instruction: String::new(),
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
    fn correspond_station(&self, inst_name: &str) -> Result<Vec<String>, String> {
        self.stations.get(inst_name)
            .map(|list| {
                let stations = list.station.borrow();
                (*stations).clone()
            }).ok_or(String::from("No comresponding station"))
    }
    pub fn decode(&mut self, inst: &str) -> Result<DecodedInst, String> {
        self.instruction = String::from(inst);
        let tokens = text_slicer(inst);
        if tokens.len() == 0 {
            let msg = format!("No token has been found in instruction {}", inst);
            return Err(msg);
        }

        let inst_name = tokens[0];
        let arguments = &tokens[1..];

        let stations = self.correspond_station(inst_name)?;

        let format = self.formats
            .get(inst_name)
            .ok_or(format!("Instruct {} has not implemented", inst))?;
        let mut args = Vec::with_capacity(tokens.len() - 1);
        let syntax = &format.syntax;
        for (token, expect_type) in arguments.iter().zip(syntax.iter()) {
            let arg = arg_scan(token)?;
            let get_type = SyntaxType::from(arg);
            if !get_type.matches(expect_type) {
                let msg = format!("Expect type {:?}, but get type {:?}", *expect_type, get_type);
                return Err(msg);
            }
            args.push(arg);
        }

        Ok(DecodedInst {
            name: inst_name.to_string(),
            stations,
            args,
            writeback: format.writeback,
        })
    }
    pub fn last_instruction(&self) -> &str {
        &self.instruction
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

#[derive(Clone, Debug)]
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
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn stations<'a>(&'a self) -> &'a Vec<String> {
        &self.stations
    }
    pub fn args<'a>(&'a self) -> &'a Vec<ArgType> {
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

#[cfg(test)]
mod syntaxtype_test {
    use super::SyntaxType;
    #[test]
    fn Sametype() {
        let a = SyntaxType::Immediate;
        let b = SyntaxType::Immediate;
        assert!(a.matches(&b));
    }
    #[test]
    fn Register_Writeback() {
        let a = SyntaxType::Register;
        let b = SyntaxType::Writeback;
        assert!(a.matches(&b));
    }
    #[test]
    fn Writeback_Register() {
        let a = SyntaxType::Writeback;
        let b = SyntaxType::Register;
        assert!(a.matches(&b));
    }
    #[test]
    fn Register_Immediate() {
        let a = SyntaxType::Register;
        let b = SyntaxType::Immediate;
        assert!(!a.matches(&b));
    }
    #[test]
    fn Writeback_Immediate() {
        let a = SyntaxType::Writeback;
        let b = SyntaxType::Immediate;
        assert!(!a.matches(&b));
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SyntaxType {
    Register,
    Writeback,
    Immediate,
}

impl SyntaxType {
    fn from(arg: ArgType) -> Self {
        match arg {
            ArgType::Reg(_) => SyntaxType::Register,
            ArgType::Imm(_) => SyntaxType::Immediate,
        }
    }
    fn matches(&self, other: &Self) -> bool {
        use SyntaxType::*;
        match (self, other) {
            (Register, Writeback) => true,
            (Writeback, Register) => true,
            (s, o) if s == o => true,
            (_, _) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstFormat {
    name: String,
    syntax: Vec<SyntaxType>,
    writeback: bool,
}

impl InstFormat {
    pub fn create(name: &str) -> InstFormatCreater {
        InstFormatCreater {
            body: InstFormat {
                name: name.to_string(),
                syntax: Vec::new(),
                writeback: false,
            },
        }
    }
}

pub struct InstFormatCreater {
    body: InstFormat,
}

impl InstFormatCreater {
    pub fn add_syntax(mut self, syn_type: SyntaxType) -> Self {
        self.body.syntax.push(syn_type);
        self
    }
    pub fn set_writeback(mut self, w: bool) -> Self {
        self.body.writeback = w;
        self
    }
    pub fn done(self) -> InstFormat {
        self.body
    }
}

