use crate::timer::Timer;
use notan::draw::*;
use notan::prelude::*;

#[derive(Debug, Clone)]
pub enum Message {
    Text(String),
    Choice(String, String),
}

pub struct Conversation {
    pub messages: Vec<Message>,
    pub current: usize,
    pub textbox: Textbox,
}

impl Conversation {
    pub fn new(messages: Vec<Message>) -> Self {
        let mut x = Conversation {
            messages,
            current: 0,
            textbox: Textbox::new(Message::Text("".to_string())),
        };

        x.textbox.message = x.current_message();
        x
    }

    pub fn setup(&mut self, gfx: &mut Graphics) {
        self.textbox.setup(gfx);
    }

    pub fn update(&mut self, delta: f32) {
        self.textbox.update(delta);
    }

    pub fn advance(&mut self) {
        self.current += 1;
        if self.current >= self.messages.len() {
            self.current = self.messages.len() - 1;
        }
        self.textbox.set_message(self.current_message());
    }

    pub fn draw(&self, draw: &mut Draw) {
        self.textbox.draw(draw);
    }

    pub fn current_message(&self) -> Message {
        self.messages[self.current].clone()
    }
}

pub struct Textbox {
    pub message: Message,
    pub cursor: usize,
    print_timer: Timer,
    font: Option<Font>,
}

impl Textbox {
    pub fn new(message: Message) -> Self {
        Textbox {
            message,
            cursor: 0,
            print_timer: Timer::new(0.03),
            font: None,
        }
    }

    pub fn set_message(&mut self, message: Message) {
        self.message = message;
        self.cursor = 0;
        self.print_timer.reset();
    }

    pub fn finished_printing(&self) -> bool {
        match self.message {
            Message::Text(ref text) => self.cursor >= text.len(),
            Message::Choice(_, _) => true,
        }
    }

    pub fn setup(&mut self, gfx: &mut Graphics) {
        let font = gfx
            .create_font(include_bytes!("assets/alagard.ttf"))
            .unwrap();

        self.font = Some(font);
    }

    pub fn advance(&mut self) -> bool {
        match &self.message {
            Message::Text(text) => {
                self.cursor += 1;
                if self.cursor > text.len() {
                    self.cursor = text.len();
                    return true;
                }
                false
            }
            Message::Choice(_, _) => false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.print_timer.update(dt);

        if self.print_timer.is_finished() {
            if !self.advance() {
                self.print_timer.reset();
            }
        }
    }

    pub fn draw(&self, draw: &mut Draw) {
        let x = 10.0;
        let mut y = 10.0;
        let max_width = 300.0;
        let font_size = 24.0;

        match &self.message {
            Message::Text(text) => {
                let wrapped_text = self.wrap_text(text, max_width, font_size);
                let visible_text: String = wrapped_text.chars().take(self.cursor).collect();

                for line in visible_text.lines() {
                    draw.text(&self.font.unwrap(), line)
                        .position(x, y)
                        .size(font_size)
                        .h_align_left()
                        .v_align_top();
                    y += font_size + 5.0; // Add some spacing between lines
                }
            }
            Message::Choice(option1, option2) => {
                draw.text(&self.font.unwrap(), option1)
                    .position(x, y)
                    .size(font_size);
                y += 30.0; // Adjust this value to set the vertical spacing between options
                draw.text(&self.font.unwrap(), option2)
                    .position(x, y)
                    .size(font_size);
            }
        }
    }

    fn wrap_text(&self, text: &str, max_width: f32, font_size: f32) -> String {
        let mut wrapped = String::new();
        let mut line = String::new();
        let avg_char_width = font_size * 0.6; // Approximate average character width

        for word in text.split_whitespace() {
            if line.len() + word.len() > (max_width / avg_char_width) as usize {
                if !line.is_empty() {
                    wrapped.push_str(&line);
                    wrapped.push('\n');
                    line.clear();
                }
                if word.len() > (max_width / avg_char_width) as usize {
                    let chars_per_line = (max_width / avg_char_width) as usize;
                    for (i, c) in word.chars().enumerate() {
                        if i > 0 && i % chars_per_line == 0 {
                            wrapped.push('\n');
                        }
                        wrapped.push(c);
                    }
                    wrapped.push('\n');
                } else {
                    line.push_str(word);
                }
            } else {
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(word);
            }
        }
        if !line.is_empty() {
            wrapped.push_str(&line);
        }
        wrapped
    }
}
