use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
/// Decoder is used to decode instruction and find
/// appropriate name of reservation station
pub struct Decoder {
    /// Last decoded instruction.
    instruction: String,
    /// Mapping between instruction and name of corresponded reservation stations.
    stations: HashMap<String, StationList>,
    /// Mapping between instruction and its syntax.
    formats: HashMap<String, InstFormat>,
}

#[cfg(test)]
mod decoder {
    use super::*;
    #[test]
    fn register_mul() {
        use TokenType::*;
        let mut d = Decoder::new();
        let test_inst = vec![
            InstFormat {
                name: String::from("add"),
                syntax: vec![Register, Register, Register],
            },
            InstFormat {
                name: String::from("addi"),
                syntax: vec![Register, Register, Immediate],
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
        use TokenType::*;
        let mut d = Decoder::new();
        let inst = vec![InstFormat {
            name: String::from("add"),
            syntax: vec![Register, Register, Immediate],
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
        use TokenType::*;
        let mut d = Decoder::new();
        let inst = vec![InstFormat {
            name: String::from("add"),
            syntax: vec![Register, Register, Immediate],
        }];
        let station = String::from("station");
        let _args = vec![
            String::from("r0"),
            String::from("R13"),
            String::from("#100"),
        ];
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
    /// Register a mapping between instruction and
    pub fn register(&mut self, inst_list: Vec<InstFormat>, station: String) -> Result<(), String> {
        if inst_list.len() == 0 {
            return Ok(());
        }

        /* Instruction implemented by different reservation stations must be disjoint or congruent.
         * For example:
         * Station1 implement [i1, i2]
         * Station2 implement [i2, i3]
         * Since Station1 and Station2 are neither disjoint nor congruent, this implementation is invalid.
         */
        if let Some(station_list) = self.stations.get_mut(&inst_list[0].name) {
            /* There is a set of stations have implemented the inserting instruction.
             * We assuem they are congruent.
             */
            /* Since all instruction of the set mapping into a same station list,
             * we dont have to do insert for every instruction mapping.
             */
            station_list.push(&station);
        } else {
            /* The instructions haven't been mapped. */
            let station = StationList::new(&station);

            /* Generate mapping for each instructions */
            for inst in inst_list.iter() {
                let name = &inst.name;
                let exist = self.stations.insert(name.clone(), station.clone());
                if let Some(_) = exist {
                    /* Some instruction has been mapped.
                     * Which means they are not disjoint, return an error
                     */
                    let msg = format!("Instruction {} has been used by other function unit", name);
                    return Err(msg);
                }
                self.formats.insert(name.clone(), inst.clone());
            }
        }
        Ok(())
    }
    /// Return name of reservation stations which are suitable to issue the instruction
    /// On found, [Ok] with vector of name of stations returned.
    /// Otherwise, [Err] with error message returned.
    fn station_of(&self, inst_name: &str) -> Result<Vec<String>, String> {
        self.stations
            .get(inst_name)
            .map(|list| {
                let stations = list.station.borrow();
                (*stations).clone()
            })
            .ok_or(String::from("Suitable reservation station not found"))
    }
    /// Decode row arguments by given syntax.
    /// On success, [Ok] with a two tuple returned.
    /// - The first entry of tuple is vector of decoded arguments
    /// - The second entry of tuple is whether the instruction writeback or not.
    ///   If writeback, it content writeback destination.
    ///
    /// Otherwise, [Err] with error message returned.
    fn decode_args(
        arguments: &[&str],
        syntax: &[TokenType],
    ) -> Result<(Vec<ArgType>, Option<ArgType>), String> {
        let mut args = Vec::with_capacity(arguments.len());
        let mut writeback = None;
        for (token, expect_type) in arguments.iter().zip(syntax.iter()) {
            let arg = arg_scan(token)?;
            let get_type = TokenType::from(arg);
            if !get_type.matches(expect_type) {
                let msg = format!(
                    "Expect type {:?}, but get type {:?}",
                    *expect_type, get_type
                );
                return Err(msg);
            }
            if let TokenType::Writeback = *expect_type {
                writeback = Some(arg);
            } else {
                args.push(arg);
            }
        }
        Ok((args, writeback))
    }
    fn syntax_of(&self, inst_name: &str) -> Result<&[TokenType], String> {
        let format = self
            .formats
            .get(inst_name)
            .ok_or(format!("Instruct {} has not implemented", inst_name))?;
        let syntax = &format.syntax;
        Ok(syntax)
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

        let stations = self.station_of(inst_name)?;
        let syntax = self.syntax_of(inst_name)?;
        let (args, writeback) = Decoder::decode_args(arguments, syntax)?;

        Ok(DecodedInst {
            name: inst_name.to_string(),
            stations,
            args,
            writeback,
        })
    }
    pub fn last_instruction(&self) -> &str {
        &self.instruction
    }
}

/// Argument scanner. Scan argument string and turn into [ArgType] (Token type).
fn arg_scan(row_arg: &str) -> Result<ArgType, String> {
    let mut chars = row_arg.chars();
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

#[test]
fn slicer_test() {
    let txt = "a b,c(d)e:";
    let slice = text_slicer(txt);
    assert_eq!(slice[0], "a");
    assert_eq!(slice[1], "b");
    assert_eq!(slice[2], "c");
    assert_eq!(slice[3], "d");
    assert_eq!(slice[4], "e");
}
/// Seperate row string to words by delimiters.
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
/// Use to record list of name of reservation stations.
struct StationList {
    station: Rc<RefCell<Vec<String>>>,
}

impl StationList {
    fn new(name: &String) -> Self {
        Self {
            station: Rc::new(RefCell::new(vec![name.clone()])),
        }
    }
    fn push(&mut self, name: &String) {
        self.station.borrow_mut().push(name.clone());
    }
}

pub struct DecodedInst {
    name: String,
    stations: Vec<String>,
    args: Vec<ArgType>,
    writeback: Option<ArgType>,
}

impl DecodedInst {
    pub fn name(&self) -> String {
        self.name.clone()
    }
    // Return a vector of name of stations which can issue the instruction.
    pub fn stations<'a>(&'a self) -> &'a Vec<String> {
        &self.stations
    }
    pub fn arguments<'a>(&'a self) -> &'a Vec<ArgType> {
        &self.args
    }
    pub fn writeback(&self) -> Option<ArgType> {
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
    use super::TokenType;
    #[test]
    fn Sametype() {
        let a = TokenType::Immediate;
        let b = TokenType::Immediate;
        assert!(a.matches(&b));
    }
    #[test]
    fn Register_Writeback() {
        let a = TokenType::Register;
        let b = TokenType::Writeback;
        assert!(a.matches(&b));
    }
    #[test]
    fn Writeback_Register() {
        let a = TokenType::Writeback;
        let b = TokenType::Register;
        assert!(a.matches(&b));
    }
    #[test]
    fn Register_Immediate() {
        let a = TokenType::Register;
        let b = TokenType::Immediate;
        assert!(!a.matches(&b));
    }
    #[test]
    fn Writeback_Immediate() {
        let a = TokenType::Writeback;
        let b = TokenType::Immediate;
        assert!(!a.matches(&b));
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TokenType {
    Register,
    Writeback,
    Immediate,
}

impl TokenType {
    fn from(arg: ArgType) -> Self {
        match arg {
            ArgType::Reg(_) => TokenType::Register,
            ArgType::Imm(_) => TokenType::Immediate,
        }
    }
    fn matches(&self, other: &Self) -> bool {
        use TokenType::*;
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
    /// Indicate the syntax of arguments, the order is matter.
    syntax: Vec<TokenType>,
}

impl InstFormat {
    pub fn create(name: &str) -> InstFormatCreater {
        InstFormatCreater {
            body: InstFormat {
                name: name.to_string(),
                syntax: Vec::new(),
            },
        }
    }
}

pub struct InstFormatCreater {
    body: InstFormat,
}

impl InstFormatCreater {
    /// Add a token type to syntax. The order of adding token type is matter.
    /// # Example
    /// Adding syntax of instruction `addi reg, reg, imm`;
    /// ```
    /// use TokenType::*;
    /// InstFormat::create("addi")
    ///     .add_syntax(Writeback)
    ///     .add_syntax(Register)
    ///     .add_syntax(Immediate)
    ///     .done()
    /// ```
    pub fn add_syntax(mut self, token_type: TokenType) -> Self {
        self.body.syntax.push(token_type);
        self
    }
    /// Always call this method after claim syntax for a instruction.
    pub fn done(self) -> InstFormat {
        self.body
    }
}
