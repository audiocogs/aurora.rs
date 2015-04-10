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
          if let ::sample_type::Unknown = audio.sample_type {
            panic!("caf::Muxer: Unknown sample type");
          }

          sink.write(|binary| {
            let d = &mut binary.data;

            d.grow(8, 0);

            std::slice::bytes::copy_memory(d.slice_mut( 0,  4), b"caff");

            unsafe {
              std::slice::bytes::copy_memory(d.slice_mut( 4,  6), std::mem::transmute::<u16, [u8; 2]>(1u16.to_be()));
              std::slice::bytes::copy_memory(d.slice_mut( 6,  8), std::mem::transmute::<u16, [u8; 2]>(0u16.to_be()));
            }
          });

          sink.write(|binary| {
            let d = &mut binary.data;

            d.grow(44, 0);
            
            std::slice::bytes::copy_memory(d.slice_mut( 0, 4), b"desc");

            unsafe {
              std::slice::bytes::copy_memory(d.slice_mut(4, 12), std::mem::transmute::<i64, [u8; 8]>(32i64.to_be()));
              
              let sample_rate = std::mem::transmute::<f64, u64>(audio.sample_rate).to_be();
              std::slice::bytes::copy_memory(d.slice_mut(12, 20), std::mem::transmute::<u64, [u8; 8]>(sample_rate));
              
              std::slice::bytes::copy_memory(d.slice_mut(20, 24), b"lpcm");
              
              let mut format_flags = 0u32;

              if let ::sample_type::Float(_) = audio.sample_type { format_flags |= 1 };
              if audio.endian == ::endian::Little { format_flags |= 2; };

              std::slice::bytes::copy_memory(d.slice_mut(24, 28), std::mem::transmute::<u32, [u8; 4]>(format_flags.to_be()));

              let bytes_per_packet = (::sample_type::size(audio.sample_type) * audio.channels / 8) as u32;

              std::slice::bytes::copy_memory(d.slice_mut(28, 32), std::mem::transmute::<u32, [u8; 4]>(bytes_per_packet.to_be()));
              std::slice::bytes::copy_memory(d.slice_mut(32, 36), std::mem::transmute::<u32, [u8; 4]>(1u32.to_be()));

              std::slice::bytes::copy_memory(d.slice_mut(36, 40), std::mem::transmute::<u32, [u8; 4]>((audio.channels as u32).to_be()));
              std::slice::bytes::copy_memory(d.slice_mut(40, 44), std::mem::transmute::<u32, [u8; 4]>((::sample_type::size(audio.sample_type) as u32).to_be()));
            }
          });

          sink.write(|binary| {
            let d = &mut binary.data;

            d.grow(12, 0);

            std::slice::bytes::copy_memory(d.slice_mut(0, 4), b"data");
            std::slice::bytes::copy_memory(d.slice_mut(4, 12), &[0xFFu8, ..8]);
          });

          first = false;
        }

        last = audio.last;

        sink.write(|binary| {
          binary.data.grow(audio.data.len(), 0);

          std::slice::bytes::copy_memory(binary.data.as_mut_slice(), audio.data.as_slice());

          binary.last = last;
        });
      });
    }
  }
}
