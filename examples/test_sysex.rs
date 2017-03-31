extern crate midir;

use std::thread::sleep;
use std::time::Duration;
use std::error::Error;

use midir::{MidiInput, MidiOutput, Ignore};
use midir::os::unix::{VirtualInput, VirtualOutput};

// TODO: better error handling using try! macro for all possible failures and printing actual error message
fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err.description())
    }
}

fn run() -> Result<(), Box<Error>> {
    let mut midi_in = try!(MidiInput::new("My Test Input"));
    midi_in.ignore(Ignore::None);
    let midi_out = try!(MidiOutput::new("My Test Output"));
    
    let previous_count = midi_out.port_count();
    
    println!("Creating virtual input port ...");
    let conn_in = try!(midi_in.create_virtual("midir-test", |stamp, message, _| {
        println!("{}: {:?} (len = {})", stamp, message, message.len());
    }, ()).map_err(|e| e.kind()));
    
    assert_eq!(midi_out.port_count(), previous_count + 1);
    
    println!("Connecting to port '{}' ...", midi_out.port_name(previous_count).unwrap());
    let mut conn_out = try!(midi_out.connect(previous_count, "midir-test").map_err(|e| e.kind()));
    println!("Starting to send messages ...");
    println!("Sending NoteOn message");
    try!(conn_out.send(&[144, 60, 1]));
    sleep(Duration::from_millis(200));
    println!("Sending small SysEx message ...");
    try!(conn_out.send(&[0xF0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xF7]));
    sleep(Duration::from_millis(200));
    println!("Sending large SysEx message ...");
    let mut v = Vec::with_capacity(2000);
    v.push(0xF0u8);
    for _ in 1..1999 {
        v.push(0u8);
    }
    v.push(0xF708);
    assert_eq!(v.len(), 2000);
    try!(conn_out.send(&v[..]));
    sleep(Duration::from_millis(200));
    println!("Sending small SysEx message ...");
    try!(conn_out.send(&[0xF0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xF7]));
    sleep(Duration::from_millis(200));
    println!("Closing output ...");
    conn_out.close();
    println!("Closing virtual input ...");
    conn_in.close().0;
    Ok(())
}