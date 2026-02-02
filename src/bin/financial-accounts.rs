use std::io::Write;

use clap::{CommandFactory, Parser};
use financial_accounts::app::{App, command_line::Args};
use iced::window;
use image::ImageFormat;

const MONEY: &[u8] = include_bytes!("financial-accounts_256x256.png");

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.man {
        let mut buffer: Vec<u8> = Vec::default();
        let cmd = Args::command()
            .name("financial-accounts")
            .long_version(None);
        let man = clap_mangen::Man::new(cmd).date("2025-12-30");

        man.render(&mut buffer)?;
        write!(buffer, "{COPYRIGHT}")?;

        std::fs::write("financial-accounts.1", buffer)?;
        return Ok(());
    }

    iced::application(App::default, App::update, App::view)
        .title("Financial Accounts")
        .window(window::Settings {
            icon: Some(iced::window::icon::from_file_data(
                MONEY,
                Some(ImageFormat::Png),
            )?),
            ..window::Settings::default()
        })
        .theme(App::theme)
        .run()?;

    Ok(())
}

pub const COPYRIGHT: &str = r#".SH COPYRIGHT
Copyright (c) 2023-2025 David Lawrence Campbell

The MIT License:

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice (including the next
paragraph) shall be included in all copies or substantial portions of the
Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

OR

The Apache-2.0 license:

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

	http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
"#;
