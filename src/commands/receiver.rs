use byteorder::{LittleEndian, ByteOrder};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::sync_channel;
use std::net::UdpSocket;
use std::thread;
use std::env;
use std::io::Cursor;

struct Receiver {
    discord_channel: Arc<Mutex<Option<Arc<SerenityMutex<Call>>>>>,
    tx: sync_channel::<Option<(Vec<u8>, u16, u16)>>
}

impl Receiver {
    pub fn new() -> Self {
        let dmr_local_rx_addr = env::var("LOCAL_RX_ADDR")
            .expect("Expected a local rx address in the environment");

        let socket = UdpSocket::bind(dmr_local_rx_addr)
            .expect("Couldn't bind udp socket for reception");

        let discord_channel = Arc::new(Mutex::new(None));

<<<<<<< HEAD
        let (tx, rx) = sync_channel::<Option<(Vec<u8>, u16, u16)>>(512);

        let channel_ref = discord_channel.clone();
        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(packet) => match packet {
                        Some((packet_data, src_id, dst_id)) => {
                            let audio = &packet_data[..packet_data.len() - 4];
                            let src_id_high_byte = packet_data[packet_data.len() - 4];
                            let src_id_low_byte = packet_data[packet_data.len() - 3];
                            let dst_id_high_byte = packet_data[packet_data.len() - 2];
                            let dst_id_low_byte = packet_data[packet_data.len() - 1];
                            let src_id = ((src_id_high_byte as u16) << 8) | (src_id_low_byte as u16);
                            let dst_id = ((dst_id_high_byte as u16) << 8) | (dst_id_low_byte as u16);

                            let mut data: [i16; 160] = [0; 160];
                            LittleEndian::read_i16_into(audio, &mut data);
                            let mut source = signal::from_iter(data.iter().cloned());
                            let first = source.next();
                            let second = source.next();
                            let interpolator = Linear::new(first, second);
                            let frames: Vec<_> = source
                                .from_hz_to_hz(interpolator, 8000.0, 48000.0)
                                .take(960)
                                .collect();
                            let mut new_data: [u8; 1920] = [0; 1920];
                            LittleEndian::write_i16_into(&frames, &mut new_data);
                            let audio_input = Input::new(
                                false,
                                Reader::from_memory(Cursor::new(Vec::from(new_data))),
                                Codec::Pcm,
                                Container::Raw,
                                None,
                            );
                            {
                                let channel: MutexGuard<Option<Arc<SerenityMutex<Call>>>> =
                                    channel_ref.lock().unwrap();
                                match &*channel {
                                    Some(device) => {
                                        let rt = Runtime::new().unwrap();
                                        let mut call = rt.block_on(async { device.lock().await });
                                        call.play_source(audio_input);
                                        let two_millis = time::Duration::from_millis(18);
                                        thread::sleep(two_millis);
                                    }
                                    None => {}
=======
        let (tx, rx) = sync_channel::<Option<Vec<u8>>>(512);

        let channel_ref = discord_channel.clone();
        thread::spawn(move || loop {
            match rx.recv() {
                Ok(packet) => match packet {
                    Some(packet_data) => {
                        let mut data: [i16; 160] = [0; 160];
                        LittleEndian::read_i16_into(&packet_data, &mut data);
                        let mut source = signal::from_iter(data.iter().cloned());
                        let first = source.next();
                        let second = source.next();
                        let interpolator = Linear::new(first, second);
                        let frames: Vec<_> = source
                            .from_hz_to_hz(interpolator, 8000.0, 48000.0)
                            .take(960)
                            .collect();
                        let mut new_data: [u8; 1920] = [0; 1920];
                        LittleEndian::write_i16_into(&frames, &mut new_data);
                        let audio = Input::new(
                            false,
                            Reader::from_memory(Vec::from(new_data)),
                            Codec::Pcm,
                            Container::Raw,
                            None,
                        );
                        {
                            let channel: MutexGuard<Option<Arc<SerenityMutex<Call>>>> =
                                channel_ref.lock().unwrap();
                            match &*channel {
                                Some(device) => {
                                    let rt = Runtime::new().unwrap();
                                    let mut call = rt.block_on(async { device.lock().await });
                                    call.play_source(audio);
                                    let two_millis = time::Duration::from_millis(18);
                                    thread::sleep(two_millis);
>>>>>>> parent of adcdd32... Update receiver.rs
                                }
                            }
                        }
                        None => return,
                    },
                    Err(_) => return,
                }
            }
        });

        let sub_tx = tx.clone();
        thread::spawn(move || {
            let mut buffer = [0u8; 352];
            loop {
                match socket.recv(&mut buffer) {
                    Ok(packet_size) => {
                        if packet_size >= 32 {
                            let packet_type_as_num = LittleEndian::read_u32(&buffer[20..24]);
                            let packet_type = match packet_type_as_num {
                                0 => {
                                    if packet_size == 32 {
                                        USRPVoicePacketType::End
                                    } else {
                                        USRPVoicePacketType::Audio
                                    }
                                }
<<<<<<< HEAD
                                2 => USRPVoicePacketType::Start,
                                _ => USRPVoicePacketType::Audio,
                            };
                            println!(
                                "[INFO] RECEIVED PACKET: {:?} (length: {}, ptt: {})",
                                packet_type,
                                packet_size,
                                BigEndian::read_u32(&buffer[12..16])
                            );
                            if packet_type == USRPVoicePacketType::Audio {
                                let audio_data = Vec::from(&buffer[32..]);
                                if audio_data.len() == 320 {
                                    let src_id = LittleEndian::read_u16(&buffer[packet_size - 4..packet_size - 2]);
                                    let dst_id = LittleEndian::read_u16(&buffer[packet_size - 2..packet_size]);
                                    if sub_tx.send(Some((audio_data, src_id, dst_id))).is_err() {
                                        return;
                                    }
                                }
                            }
=======
                            }
                            2 => USRPVoicePacketType::Start,
                            _ => USRPVoicePacketType::Audio,
                        };
                        println!(
                            "[INFO] RECEIVED PACKET: {:?} (length: {}, ptt: {})",
                            packet_type,
                            packet_size,
                            BigEndian::read_u32(&buffer[12..16])
                        );
                        if packet_type == USRPVoicePacketType::Audio {
                            let audio = Vec::from(&buffer[32..]);
                            if audio.len() == 320 && sub_tx.send(Some(audio)).is_err() { return }
>>>>>>> parent of adcdd32... Update receiver.rs
                        }
                    }
                    Err(_) => return,
                }
            }
        });

        Self {
            discord_channel,
            tx,
        }
    }
}
