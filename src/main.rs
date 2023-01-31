#![windows_subsystem = "windows"] 

// iced for GUI-related things
use iced::widget::{Row, button, container, column, row, text, text_input, scrollable};
use iced::{Alignment, Color, Element, Font, Length, Sandbox, Settings};
use iced::theme::{Theme};

// file i/o stuff lol
use std::fs::File;
use std::io::prelude::*;

// syntect for parsing and highlighting with the .sublime-syntax file.
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use std::path::Path;
use syntect::highlighting::{ThemeSet, Style};
use syntect::highlighting::Color as OtherColor;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};
use iced::widget::text::Text;
// external modules used in creating structures for the simulator
mod legv8;
mod registers;
use registers::registers as regs;
use crate::regs::registers;
use crate::legv8::Instruction;

pub fn main() -> iced::Result {
    Simulator::run(Settings::default())
}

fn readfile(fname: &str) -> std::io::Result<String>{
    let mut file = File::open(fname.to_string().trim())?;
    let mut code = String::new();
    file.read_to_string(&mut code)?;
    Ok(code)
}

fn parse(code: &str)-> (Vec<Style>, Vec<String>){
    let ss = SyntaxSet::load_from_folder("src/syntax/legv8.sublime-syntax").unwrap();
    let ts = ThemeSet::load_defaults();
    let syntax = ss.find_syntax_by_extension("s").unwrap_or_else(||ss.find_syntax_plain_text());
    let mut h = HighlightLines::new(syntax, &ts.themes["Solarized (dark)"]);
    let mut sty: Vec<Style> = Vec::new();
    let mut stat: Vec<&str> = Vec::new();
    for line in LinesWithEndings::from(code) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ss).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        let (mut sty1, mut stat1): (Vec<Style>, Vec<&str>) = ranges.into_iter().unzip();
        sty1.push(sty1.last().cloned().unwrap());
        stat1.push("\n");
        sty.append(&mut sty1);
        stat.append(&mut stat1);
        
        print!("{}", escaped);
    }
    print!("\n");
    let statstr: Vec<String> = stat.iter().map(|s| s.to_string()).collect();
    return (sty, statstr);
}

struct Simulator<'a>{
   regs: Vec<registers::Reg>,
   instructions: Vec<Instruction>,
   main_mem: Vec<f32>,
   st: String,
   darkmode: bool,
   code: String,
   codes: Vec<Text<'a>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Input(String),
    FileOpen,
    ThemeChange
}

impl Sandbox for Simulator<'_>{
    type Message = Message;
    fn new() -> Self {
        let mut a = Vec::new();
        for i in 0..32 {
            a.push(registers::Reg{val: 0.0, name: format!("X{}", i)})
        }
        Self { regs: a, main_mem:Vec::new(), instructions:Vec::new(), darkmode:true,
        st:"".to_string(), code:"".to_string(), codes: Vec::new()}
        
    }
     
    fn title(&self) -> String {
        String::from("LEGV8 Simulator")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Input(s) => {
                self.st = s;
            }
            Message::FileOpen => {
                let result = readfile(&self.st);
                self.code = match result {
                    Ok(ref val) => val.to_string(),
                    Err(ref _err) => "Error reading your file.".to_string()
                };
                let v: Vec<&str> = self.st.split('.').collect();
                if v.len() != 2 || v[1].ne("s"){
                    self.code = "Please use a .s assembly file to simulate.".to_string();
                }
                self.codes.clear();
                if result.is_ok(){
                    let styles: (Vec<Style>, Vec<String>) = parse(&self.code);
                    for x in 0..(styles.0).len()  {
                        let a: OtherColor = styles.0[x].foreground;
                        self.codes.push(Text::new(styles.1[x].clone()).style(Color::from_rgb((a.r as f32)/255.0, (a.g as f32)/255.0, (a.b as f32)/255.0)).into());
                    }

                }
                else {
                    self.codes.push(Text::new(self.code.clone()).into());
                }
                
                
                self.regs[0].val += 10.5;
            }
            Message::ThemeChange => {
                self.darkmode = !self.darkmode;
            }

        }
    }

    fn view(&self) -> Element<Message> {
        const BOLD_FONT: Font = Font::External { 
            name: "bold font",
            bytes: include_bytes!("resources/Lato-Black.ttf")};  
        let mut a: Row<Message> = Row::new();
        for i in 0..self.codes.len() {
            a = a.push(row![self.codes[i].to_owned()]);
        }
        let content: Element<_> = container(a.align_items(Alignment::Start).padding(30)).width(Length::Fill).into();
        
        Element::from(column![column![
            row![text("File viewer").font(BOLD_FONT).size(30),button("Toggle Theme").on_press(Message::ThemeChange)].spacing(10).align_items(Alignment::Center), 
            row![text("Name of file to be simulated:").size(20)].align_items(Alignment::Center),
            row![text_input(&String::new(), &self.st, Message::Input), 
            button("Ok").on_press(Message::FileOpen)].align_items(Alignment::Center)].padding(30),container(scrollable(content)).height(Length::FillPortion(3)), 
            row![registers(self.regs.clone()), text("memory placeholder lol")]].height(Length::FillPortion(3)).width(Length::Fill).padding(20))
    }
    fn theme(&self) -> Theme {
        if self.darkmode {
            Theme::Dark 
        }
        else {
            Theme::Light
        }
    }
}