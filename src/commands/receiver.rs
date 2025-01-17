use byteorder::{BigEndian, ByteOrder, LittleEndian};
use dasp_interpolate::linear::Linear;
use dasp_signal::{self as signal, Signal};
use serenity::prelude::Mutex as SerenityMutex;
use songbird::input::{Codec, Container, Reader};
use songbird::{input::Input, Call};
use std::net::UdpSocket;
use std::time::{Duration, Instant};
use reqwest::{Client, Request};

use std::sync::{
    mpsc::{sync_channel, SyncSender},
    Arc, Mutex, MutexGuard,
};
use std::thread;
use std::{env, time};
use tokio::runtime::Runtime;
use dmr_bridge_discord::USRPVoicePacketType;

pub struct Receiver {
    discord_channel: Arc<Mutex<Option<Arc<SerenityMutex<Call>>>>>,
    tx: SyncSender<Option<Vec<u8>>>,
}

impl Drop for Receiver {
    fn drop(&mut self) {
        self.tx.send(None).unwrap();
    }
}

impl Receiver {
    pub fn new() -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.
        let dmr_local_rx_addr = env::var("LOCAL_RX_ADDR")
            .expect("Expected a local rx address in the environment");

        let socket = UdpSocket::bind(dmr_local_rx_addr)
            .expect("Couldn't bind udp socket for reception");

        let discord_channel = Arc::new(Mutex::new(None));

        let (tx, rx) = sync_channel::<Option<Vec<u8>>>(512);

        let channel_ref = discord_channel.clone();
        thread::spawn(move || {
            let mut playback_ended = false;

            while !playback_ended {
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
                                    }
                                    None => {}
                                }
                            }
                        }
                        None => {
                            playback_ended = true;
                            println!("Playback ended");
                        }
                    },
                    Err(_) => return,
                }
            }
        });

        let sub_tx = tx.clone();
        let mut first_packet_received = false;
        let mut previous_audio_end = time::Instant::now();
        let mut last_heard = 0;

        thread::spawn(move || {
            let mut buffer = [0u8; 352];
            let mut audio_buffer = Vec::new();
            let mut consecutive_packet_counter = 0;

            loop {
                match socket.recv_from(&mut buffer) {
                    Ok((packet_size, _)) => {
                        consecutive_packet_counter = 0;

                        if packet_size >= 4 {
                            let src_id =
                                u16::from_be_bytes([buffer[packet_size - 4], buffer[packet_size - 3]]);
                            let dst_id =
                                u16::from_be_bytes([buffer[packet_size - 2], buffer[packet_size - 1]]);
                            let audio_data = &buffer[..(packet_size - 4)];

                            if last_heard != src_id {
                                println!(
                                    "[INFO] RECEIVED PACKET: (length: {}, src_id: {}, dst_id: {})",
                                    packet_size,
                                    src_id,
                                    dst_id
                                );
                             //   send_discord_webhook(&src_id.to_string(), &dst_id.to_string());
                                first_packet_received = true;
                            }

                            if audio_data.len() == 320 {
                                // Append the received audio to the buffer
                                audio_buffer.extend_from_slice(audio_data);
                                last_heard = src_id;
                                // Check if enough audio samples are accumulated for playback
                                while audio_buffer.len() >= 320 {
                                    // Extract a chunk of 320 samples for playback
                                    let audio_chunk =
                                        audio_buffer.drain(..320).collect::<Vec<u8>>();

                                    // Calculate the time offset for smooth playback
                                    let current_audio_end =
                                        previous_audio_end + time::Duration::from_micros(320);

                                    // Simulate the playback delay if necessary
                                    let now = time::Instant::now();
                                    if now < current_audio_end {
                                        thread::sleep(current_audio_end - now);
                                    }

                                    // Send the audio chunk for playback
                                    if sub_tx.send(Some(audio_chunk)).is_err() {
                                        return;
                                    }

                                    // Update the previous audio end position
                                    previous_audio_end = current_audio_end;
                                }
                            }
                        }
                    }
                    Err(_) => return,
                }

                // Increment the consecutive packet counter
                consecutive_packet_counter += 1;

                // Check if consecutive packets have not been received for a certain number of iterations
                if consecutive_packet_counter > 10 {
                    println!("Playback ended");
                    break; // Exit the loop when playback is finished
                }
            }
        });

        Self {
            discord_channel,
            tx,
        }
    }

    pub fn set(&mut self, device: Arc<SerenityMutex<Call>>) {
        let device = Arc::clone(&device);
        let mut discord_channel = self.discord_channel.lock().unwrap();
        *discord_channel = Some(device);
    }

    pub fn unset(&mut self) {
        let mut discord_channel = self.discord_channel.lock().unwrap();
        *discord_channel = None;
    }
}

//async fn send_discord_webhook() -> Result<(), Box<dyn std::error::Error>> {
//    let request = client.post("https://discord.com/api/webhooks/1128288295068110909/VlmFL3xYoQW5cpv_Otuxn2D1hgD3N-V_0TTp4lt2Z8OUaI0Zvi9ElwMCsW2-u0Oq4ya4").body("data");
//
//    if let Err(e) = request.send().await {
//        // Handle the error...
 //   }
//
 //   let client = Client::new();
 //   let payload = format!(
 //       r#"{{"content": "[INFO] RECEIVED PACKET: (src_id: {}, dst_id: {})"}}"#,
 //       src_id, dst_id
  //  );
  //  let request = client
  //      .post(webhook_url)
  //      .body(payload)
   //     .header("Content-Type", "application/json");
   // Ok(()) 
