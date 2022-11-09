use std::thread;
use std::sync::mpsc;

const VERSION: &str = "0.1";
const BORDER_FRAME: u8 = 0x7E;
const MODULE_NUMBER: u8 = 0xF;

// const FRAME: Vec<u8> = vec![0x7E, 0x0F, 0xBE, 0x0, 0x7E]; // Missing checksum

#[derive(Default)]
struct RS232Serial
{
}

impl RS232Serial
{
    // Spawns a thread that eats messages and determines if the checksum is real
    fn start_reciever(&self, sender: mpsc::Sender<u8>) -> bool
    {
        // Wait to get bytes from the serial device
        // Push new message message to responder

        //Spawning thread so as to ensure we grab the next frame as soon as possible
        thread::spawn(move|| {
            let test_frame: Vec<u8> = vec![0x7E, 0x0, 0x0, 0x0, 0x7E];
            loop 
            {
                test_frame.iter().for_each(|byte| sender.send(*byte).expect("Failed to send byte to parser"));
            }
        });
        true
    }

    //send the response back across the serial device
    fn send_response(&self, response: Vec<u8>) -> bool
    {
        response.iter().for_each(|byte| println!("Got {}", byte));
        true
    }
}

// Frame Logic, 

enum ParseError
{
    BadChecksum,
    InvalidFrame,
}

fn build_response(frame: Vec<u8>) -> Result<Vec<u8>, ParseError>
{
    // fcs and compare with recieved frame
    let calculated_checksum = fcs(&frame[1..(frame.len()-3)]);
    let found_checksum: u16 = frame[(frame.len()-3)..(frame.len()-1)].iter().fold(0, |cs: u16, byte| {cs << 8; cs + (*byte as u16)});
    if calculated_checksum != found_checksum 
    {
        return Err(ParseError::BadChecksum);
    }
    if frame[1] != MODULE_NUMBER 
    {
        return Err(ParseError::InvalidFrame);
    }
    // Parse command as valid or not
    // if not valid, discard frame (emulating real hardware)
    // craft preset response based on message type
    Ok(vec![0x00, 0x00])
}

fn fcs(frame: &[u8]) -> u16
{
    42
}

fn main() {
    println!("SPI Laser Emulator Version {}", VERSION);
    let (sender, reciever) = mpsc::channel();
    let serial = RS232Serial::default();
    serial.start_reciever(sender);
    loop
    {
        let mut frame_count = 0;
        let mut new_frame = Vec::<u8>::new();
        while frame_count != 2
        {
            let new_byte: u8 = reciever.recv().unwrap();
            if new_byte == BORDER_FRAME
            {
                frame_count += 1;
            }
            if frame_count > 0
            {
                new_frame.push(new_byte);
            }
        }
        if let Ok(response) = build_response(new_frame)
        {
            serial.send_response(response);
        }
    }
}
