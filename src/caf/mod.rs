use std;

use channel;

pub struct Muxer {
  source: channel::Source<::Audio>,
  sink: channel::Sink<::Binary>
}

impl Muxer {
  pub fn new(source: channel::Source<::Audio>, sink: channel::Sink<::Binary>) -> Muxer {
    return Muxer {
      source: source,
      sink: sink
    };
  }

  pub fn run(&mut self) {
    let mut first = true;
    let mut last = false;

    let source = &mut self.source;
    let sink = &mut self.sink;

    while !last {
      source.read(|audio| {
        if first {
          if let ::sample_type::SampleType::Unknown = audio.sample_type {
            panic!("caf::Muxer: Unknown sample type");
          }

          sink.write(|binary| {
            let d = &mut binary.data;

            d.reserve(8);

            std::slice::bytes::copy_memory(b"caff", &mut d[0..4]);

            unsafe {
              std::slice::bytes::copy_memory(&std::mem::transmute::<u16, [u8; 2]>(1u16.to_be()), &mut d[4..6]);
              std::slice::bytes::copy_memory(&std::mem::transmute::<u16, [u8; 2]>(0u16.to_be()), &mut d[6..8]);
            }
          });

          sink.write(|binary| {
            let d = &mut binary.data;

            d.reserve(44);
            
            std::slice::bytes::copy_memory(b"desc", &mut d[0..4]);

            unsafe {
              std::slice::bytes::copy_memory(&std::mem::transmute::<i64, [u8; 8]>(32i64.to_be()), &mut d[4..12]);
              
              let sample_rate = std::mem::transmute::<f64, u64>(audio.sample_rate).to_be();
              std::slice::bytes::copy_memory(&std::mem::transmute::<u64, [u8; 8]>(sample_rate), &mut d[12..20]);
              
              std::slice::bytes::copy_memory(b"lpcm", &mut d[20..24]);
              
              let mut format_flags = 0u32;

              if let ::sample_type::SampleType::Float(_) = audio.sample_type { format_flags |= 1 };
              if audio.endian == ::endian::Endian::Little { format_flags |= 2; };

              std::slice::bytes::copy_memory(&std::mem::transmute::<u32, [u8; 4]>(format_flags.to_be()), &mut d[24..28]);

              let bytes_per_packet = (::sample_type::size(audio.sample_type) * audio.channels / 8) as u32;

              std::slice::bytes::copy_memory(&std::mem::transmute::<u32, [u8; 4]>(bytes_per_packet.to_be()), &mut d[28..32]);
              std::slice::bytes::copy_memory(&std::mem::transmute::<u32, [u8; 4]>(1u32.to_be()), &mut d[32..36]);

              std::slice::bytes::copy_memory(&std::mem::transmute::<u32, [u8; 4]>((audio.channels as u32).to_be()), &mut d[36..40]);
              std::slice::bytes::copy_memory(&std::mem::transmute::<u32, [u8; 4]>((::sample_type::size(audio.sample_type) as u32).to_be()), &mut d[40..44]);
            }
          });

          sink.write(|binary| {
            let d = &mut binary.data;

            d.reserve(12);

            std::slice::bytes::copy_memory(b"data", &mut d[0..4]);
            std::slice::bytes::copy_memory(&[0xFFu8; 8], &mut d[4..12]);
          });

          first = false;
        }

        last = audio.last;

        sink.write(|binary| {
          binary.data.reserve(audio.data.len());

          std::slice::bytes::copy_memory(&audio.data, &mut binary.data);

          binary.last = last;
        });
      });
    }
  }
}
