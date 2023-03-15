use std::thread;
use std::sync::mpsc;
use serial;

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
    fn start_reciever(&self, sender: mpsc::Sender<u8>)
    {
        // Wait to get bytes from the serial device
        // Push new message message to responder
        thread::spawn(move|| {
            let test_frame: Vec<u8> = vec![0x7E, 0x0, 0x0, 0x0, 0x7E];
            loop 
            {
                test_frame.iter().for_each(|byte| sender.send(*byte).expect("Failed to send byte to parser"));
            }
        });
    }

    fn start_responder(&self, reciever: mpsc::Receiver<u8>)
    {
        // Wait to get bytes from the serial device
        // Push new message message to responder

        //Spawning thread so as to ensure we grab the next frame as soon as possible
        thread::spawn(move|| {
            let mut response_frame = Vec::<u8>::new();
            let mut frame_bytes: u8;
            loop
            {
                response_frame.clear();
                frame_bytes = 0;
                while frame_bytes < 2
                {
                    response_frame.push(reciever.recv().unwrap());
                    if response_frame[response_frame.len() - 1] == BORDER_FRAME
                    {
                        frame_bytes += 1
                    }
                }

                // send frame over serial tx
                RS232Serial::send_response(&response_frame);
            }
        });
    } 

    //send the response back across the serial device
    fn send_response(response: &Vec<u8>) -> bool
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

fn build_response(frame: &Vec<u8>) -> Result<Vec<u8>, ParseError>
{
    // fcs and compare with recieved frame
    let calculated_checksum = fcs(&frame[1..(frame.len()-4)]);
    let found_checksum: u16 = frame[(frame.len()-3)..(frame.len()-2)].iter().fold(0, |cs: u16, byte| {let cs = cs << 8; cs + (*byte as u16)});
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

// From the SPI Laser User Guide, ported to Rust
fn fcs(command: &[u8]) -> u16
{
    let polynomial: u16 = 0x1081;
    let nibble_mask: u16 = 0xF;
    let mut temp_sum: u16;
    let mut checksum: u16 = 0xFFFF;
    for count in 0..command.len()
    {
        // subtract first nibble from checksum
        temp_sum = (checksum ^ command[count] as u16) & nibble_mask;
        checksum >>= 4;
        checksum = checksum ^ (temp_sum * polynomial);
        temp_sum = (checksum ^ ((command[count] >> 4) as u16)) & nibble_mask;
        checksum >>= 4;
        checksum = checksum ^ (temp_sum * polynomial);
    }
    // CRC finalized by inverting all the bits
    checksum ^ 0xFFFF
}

fn process_frames(tx_sender: mpsc::Sender<u8>, rx_reciever: mpsc::Receiver<u8>)
{
    let mut new_frame = Vec::<u8>::new();
    loop
    {
        let mut frame_count: u8 = 0;
        new_frame.clear();
        while frame_count != 2
        {
            let new_byte: u8 = rx_reciever.recv().unwrap();
            if new_byte == BORDER_FRAME
            {
                frame_count += 1;
            }
            if frame_count > 0
            {
                new_frame.push(new_byte);
            }
        }
        if let Ok(response) = build_response(&new_frame)
        {
            response.iter().for_each(|byte| tx_sender.send(*byte).expect("Failed to send byte to responser"));
        }
    }
}

fn main() {
    println!("SPI Laser Emulator Version {}", VERSION);

    // Messages for the reciever
    let (rx_sender, rx_reciever) = mpsc::channel();
    // Messages for the sender
    let (tx_sender, tx_reciever) = mpsc::channel();

    let serial = RS232Serial::default();
    serial.start_reciever(rx_sender);
    serial.start_responder(tx_reciever);
    // process incoming frames and, if valid, send them to the responder
    process_frames(tx_sender, rx_reciever);
}
