use gen_iter::GenIter;

use crate::{notes::MIDINote, num::MIDINum, unwrap, yield_error};

/// Merge an array of note iterators together into one iterator.
pub fn merge_notes_array<
    T: MIDINum,
    N: MIDINote<T>,
    Err,
    I: Iterator<Item = Result<N, Err>> + Sized,
>(
    array: Vec<I>,
) -> impl Iterator<Item = Result<N, Err>> {
    struct SeqTime<T: MIDINum, N: MIDINote<T>, Err, I: Iterator<Item = Result<N, Err>> + Sized> {
        iter: I,
        time: T,
        next: Option<N>,
    }

    GenIter(move || {
        let mut seqences = Vec::new();
        for mut seq in array.into_iter() {
            let first = seq.next();
            match first {
                None => continue,
                Some(e) => match e {
                    Err(e) => yield_error!(Err(e)),
                    Ok(e) => {
                        let s = SeqTime {
                            time: e.start(),
                            next: Some(e),
                            iter: seq,
                        };
                        seqences.push(s);
                    }
                },
            }
        }

        while seqences.len() > 0 {
            let len = seqences.len();
            let mut smallest_index = 0;
            let mut smallest_time = seqences[0].time;
            for i in 0..len {
                let next = &seqences[i];
                if next.time < smallest_time {
                    smallest_time = next.time;
                    smallest_index = i;
                }
            }
            loop {
                let (note, next) = {
                    let smallest = &mut seqences[smallest_index];

                    let note = smallest.next.take().unwrap();

                    (note, smallest.iter.next())
                };
                yield Ok(note);
                match next {
                    None => {
                        seqences.remove(smallest_index);
                        break;
                    }
                    Some(next) => {
                        let next = unwrap!(next);
                        let mut smallest = &mut seqences[smallest_index];
                        smallest.time = next.start();
                        smallest.next = Some(next);
                    }
                }
                if seqences[smallest_index].time != smallest_time {
                    break;
                }
            }
        }
    })
}
