extern crate argparse;
use argparse::{ArgumentParser, Store};

extern crate tiny_http;

use std::str::FromStr;
use std::fs::{self, File};
use std::path::{self};
use std::vec::Vec;
    
struct Config {
	port_number: u16,
	working_dir: path::PathBuf,
}    
    
fn main() {
	
	let config = init_config();
	match check_config(&config) {
		Ok(_) => {}
		Err(e) => {
			println!("{}", e);
			return;
		}
	}
	
	let server = tiny_http::ServerBuilder::new().with_port(config.port_number).build().unwrap();
	println!("server listening on port {}", config.port_number);
	loop {
	    
	    match server.recv() {
	        Ok(rq) => handle_request(&config, rq),
	        Err(e) => { println!("error: {}", e); break }
	    };
	}
}

fn init_config() -> Config {
	
	let mut port_number: u16 = 8080;
	let mut working_dir = ".".to_string();
	
	{
		let mut ap = ArgumentParser::new();
		ap.set_description("simple HTTP static server");
		
		ap.refer(&mut port_number)
	            .add_option(&["-p", "--port"], Store,
	            "the port number");
	    ap.refer(&mut working_dir)
			    .add_option(&["-d", "--dir"], Store,
			    "the working directory");
	        
		ap.parse_args_or_exit();
	}
	
	let mut working_dir_path_buf: path::PathBuf;
	
	if working_dir.starts_with(".") {
		
		working_dir_path_buf = std::env::current_dir().unwrap();
		//we remove "."
		working_dir.remove(0);
		if !working_dir.is_empty() {
			working_dir_path_buf.push(working_dir); 
		}
	} else {
		working_dir_path_buf = path::PathBuf::from(working_dir);
	}
	
	println!("port_number={}, working_dir={}", port_number, working_dir_path_buf.to_str().unwrap());
	
    Config{port_number: port_number, working_dir: working_dir_path_buf}
}

fn check_config(config: &Config) ->  Result<(), &str> {

	match fs::metadata(&config.working_dir) {
		Ok(working_dir_metadata) => {
		
			if !working_dir_metadata.is_dir() {
				return Err("not a working directory");
			}
			
			
			
		}
		Err(_) => {
/*		
			let err_msg: &str = &fmt::format(format_args!("error accessing working directory {}", 
				config.working_dir));
*/		
			return Err("error accessing working directory");
		}
	}

	Result::Ok(())
}

fn handle_request(config: &Config, request: tiny_http::Request) {

	println!("request -> http_version: {}, method: {}, url: {}", 
		request.http_version(), request.method(), request.url());
	
	let mut is_dir = false;
	let mut path_str: String;
	{
		let (path, request) = url_to_path(config, request);
		match path.to_str() {
			
			Some(path) => {
				
				path_str = String::new() + path;
				
				match fs::metadata(path) {
					Ok(metadata) => {
						
						if metadata.is_dir() {
							is_dir = true;
						} 
					}
					Err(_) => {}
				}
			}
			None => {
				return;
			}
		}
			
		if is_dir {
			list_dir(config, request, &path_str);
		} else {
			send_file(request, &path_str);
		}
	}
		
}

fn url_to_path(config: &Config, request: tiny_http::Request) -> (path::PathBuf, tiny_http::Request) {

	let mut path_builder: path::PathBuf;
	{
		let req_url_tokens = request.url().split("/");
			
		path_builder = path::PathBuf::from(&config.working_dir);
		for token in req_url_tokens {
			path_builder.push(token);
		}
			
		println!("path_builder={:?}", path_builder);
	}
	
	(path_builder, request)
}

fn send_file(request: tiny_http::Request, path: &str) {

	let content_type_h: tiny_http::Header = FromStr::from_str("Content-Type:application/octet-stream").unwrap();

	match File::open(path) {
		
		Ok(file) => {
		
			let response = tiny_http::Response::from_file(file)
				.with_header(content_type_h);
	
			request.respond(response);

		},
		Err(_) => {}
	}
	
}

fn list_dir(config: &Config, request: tiny_http::Request, path: &str) {

	let mut html_output: String = "<!DOCTYPE html><html><title>rust static server</title></head><body><ul>".to_string();

	match fs::read_dir(path) {
		
		Ok(dir) => {
			
			for dir_entry in dir {
				
				match dir_entry {
					Ok(dir_entry) => {
						html_output = html_output + &dir_entry_to_html(config, &dir_entry);
					}
					Err(_) => {}
				}
			}
		
		}
		Err(_) => {}
	}
	
	html_output = html_output + "</ul></body></html>";
	
	let content_type_h: tiny_http::Header = FromStr::from_str("Content-Type:text/html").unwrap();

	let response = tiny_http::Response::from_string(html_output)
		.with_header(content_type_h);
	
	request.respond(response);
}

fn dir_entry_to_html(config: &Config, dir_entry: &fs::DirEntry) -> String {
	
	let mut html: String;
	
	match dir_entry.metadata() {
		Ok(metadata) => {
			
			html = "<li".to_string();
			if metadata.is_dir() {
				html = html + " style=\"font-weight: bold;\"";
			}
			html = html + ">";
			
			html = html + &dir_entry_to_relative_url(config, dir_entry) +
				 "</li>";
			
		}
		Err(_) => {
			html = "".to_string();
		}
	}
	
	html
}

fn dir_entry_to_relative_url(config: &Config, dir_entry: &fs::DirEntry) -> String {

	let mut path_token_vec: Vec<std::ffi::OsString> = Vec::new();
	
	for path_token in dir_entry.path().iter() {
	
		let mut s = std::ffi::OsString::new();
		s.push(path_token);
		
		path_token_vec.push(s);
	}

	let mut i = 0;
	for path_token in config.working_dir.iter() {
		
		let mut s = std::ffi::OsString::new();
		s.push(path_token);
		
		if s != path_token_vec[i] {
			break;
		}
		
		i = i + 1;
	}
	
	let mut relative_url_str = String::new();
	let path_token_vec_len = path_token_vec.len();
	while i < path_token_vec_len {
		
		relative_url_str.push('/');
		
		let token_copy: std::ffi::OsString = path_token_vec[i].clone();
		let token_copy_as_str = token_copy.into_string().unwrap();
		
		relative_url_str.push_str(&token_copy_as_str);
			
		i = i + 1;
	}

	let mut url = String::new();
	
	url = url + "<a href=\"." + &relative_url_str +  
		"\">" + dir_entry.file_name().to_str().unwrap() + "</a>";
	
	url
}




