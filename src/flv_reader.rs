use anyhow::{anyhow, bail, Result};
use flavors::parser::{self as flv, tag_data, TagHeader};
use nom::{error::ErrorKind, number::complete::be_u32, IResult};
use std::io::{BufReader, Read, Seek};

struct FlvReader<R>
where
    R: Read + Seek,
{
    reader: BufReader<R>,
    has_video: bool,
    has_audio: bool,
}

impl<R> FlvReader<R>
where
    R: Read + Seek,
{
    fn new(reader: R) -> Self {
        FlvReader {
            reader: BufReader::new(reader),
            has_audio: false,
            has_video: false,
        }
    }

    fn read_header(&mut self) -> Result<()> {
        let mut header = vec![0u8; 9];
        // read header size
        self.reader.read_exact(header.as_mut_slice())?;

        match flv::header(header.as_slice()) {
            Ok((_, header)) => {
                println!("{:?}", header);
                self.has_audio = header.audio;
                self.has_video = header.video;
                self.reader
                    .seek(std::io::SeekFrom::Start(header.offset as u64))?;
            }
            Err(_) => todo!(),
        }

        Ok(())
    }

    fn read_tag(&mut self) -> Result<()> {
        let mut tag = vec![0u8; 15];
        self.reader.read_exact(tag.as_mut_slice())?;

        // Get previous tag size
        let prev_tag_size = be_u32::<_, (&[u8], ErrorKind)>(&tag[..4])
            .map(|(_, tag_size)| tag_size)
            .map_err(|e| anyhow!("{:#?}", e))?;

        println!("Previous tag size: {}", prev_tag_size);

        // Decode tag header
        let header = match flv::tag_header(&tag[4..]) {
            Ok((_, header)) => Ok::<_, anyhow::Error>(header),
            Err(e) => {
                bail!("{:#?}", e);
            }
        }?;

        // Read the rest of the tag
        tag.resize(15 + header.data_size as usize, 0u8);
        self.reader.read_exact(&mut tag[15..])?;

        let (_, tag) = flv::complete_tag(&tag[4..]).map_err(|e| e.to_owned())?;
        println!("{:#?}", tag);
        // match header.tag_type {
        //     flv::TagType::Video => {
        //         let mut data = vec![0u8; header.data_size as usize];
        //         {
        //             self.reader.read_exact(data.as_mut_slice());
        //             let data_parser = tag_data(header.tag_type, header.data_size as usize);
        //             let (_, video_tag_data) = data_parser(&data)?;
        //             match video_tag_data {
        //                 flv::TagData::Video(v) => {}
        //                 _ => {}
        //             }
        //         }
        //     }
        //     _ => {}
        // }

        Ok(())
    }

    fn parse(&mut self) -> Result<()> {
        self.read_header()?;

        self.read_tag()?;
        self.read_tag()?;
        self.read_tag()?;
        self.read_tag()?;
        self.read_tag()?;
        self.read_tag()?;
        self.read_tag()?;
        self.read_tag()?;
        self.read_tag()?;
        self.read_tag()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_avc_avc_avc_eflv() -> Result<()> {
        let file = std::fs::File::open("corpus/ertmp-avc-avc-avc.flv")?;
        let mut byte_reader = FlvReader::new(file);

        let bytes = byte_reader.parse()?;

        Ok(())
    }

    #[test]
    fn load_av1_hevc_avc_eflv() -> Result<()> {
        let file = std::fs::File::open("corpus/ertmp-av1-hevc-avc.flv")?;
        let mut byte_reader = FlvReader::new(file);

        let bytes = byte_reader.parse()?;

        Ok(())
    }
}
