use std::cmp;
use std::collections;
use std::io;

#[derive(Default)]
pub struct Heap<T>
where
    T: io::Read,
{
    heap: collections::BinaryHeap<Entry<T>>,
}

impl<T> Heap<T>
where
    T: io::Read,
{
    pub fn new() -> Heap<T> {
        let heap = collections::BinaryHeap::new();
        Heap { heap }
    }

    pub fn add_reader(&mut self, filename: String, reader: T) -> io::Result<Option<String>> {
        let buf_reader = io::BufReader::new(reader);
        self.readd_reader(filename, buf_reader)
    }

    fn readd_reader(
        &mut self,
        filename: String,
        mut buf_reader: io::BufReader<T>,
    ) -> io::Result<Option<String>> {
        let mut first_line = String::new();
        let n = io::BufRead::read_line(&mut buf_reader, &mut first_line)?;
        if n > 0 {
            let first_line = first_line.trim_end().to_string();
            self.heap.push(Entry {
                filename,
                reader: buf_reader,
                first_line: first_line.clone(),
            });
            Ok(Some(first_line))
        } else {
            Ok(None)
        }
    }

    pub fn print_sorted_lines(&mut self) -> io::Result<()> {
        for line in self {
            if let Ok(contents) = line {
                println!("{}", contents);
            } else {
                line?;
            }
        }
        Ok(())
    }
}

impl<T> Iterator for Heap<T>
where
    T: io::Read,
{
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<io::Result<String>> {
        if let Some(Entry {
            filename,
            reader,
            first_line,
        }) = self.heap.pop()
        {
            let next_line_result = self.readd_reader(filename.clone(), reader);
            match next_line_result {
                Ok(next_line) => {
                    if next_line.is_some() && next_line.unwrap() < first_line {
                        Some(Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("Input lines in file [{}] out of order!", filename),
                        )))
                    } else {
                        Some(Ok(first_line))
                    }
                }
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct Entry<T>
where
    T: io::Read,
{
    filename: String,
    reader: io::BufReader<T>,
    first_line: String,
}

impl<T> PartialEq for Entry<T>
where
    T: io::Read,
{
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
    }
}

impl<T> Eq for Entry<T> where T: io::Read {}

impl<T> Ord for Entry<T>
where
    T: io::Read,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        if self == other {
            cmp::Ordering::Equal
        } else {
            cmp::Ordering::reverse(self.first_line.cmp(&other.first_line))
        }
    }
}

impl<T> PartialOrd for Entry<T>
where
    T: io::Read,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(clippy::string_lit_as_bytes)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single() -> Result<(), io::Error> {
        let mut heap = Heap::new();
        heap.add_reader("file1".to_string(), "bar\nfoo".as_bytes())?;
        assert_eq!(heap.next().unwrap()?, "bar");
        assert_eq!(heap.next().unwrap()?, "foo");
        assert!(heap.next().is_none());
        Ok(())
    }

    #[test]
    fn test_single_ooo() -> Result<(), io::Error> {
        let mut heap = Heap::new();
        heap.add_reader("file1".to_string(), "foo\nbar".as_bytes())?;
        let err = heap.next().unwrap().expect_err("Expected an error");
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(
            format!("{}", err),
            "Input lines in file [file1] out of order!"
        );
        Ok(())
    }

    #[test]
    fn test_multiple() -> Result<(), io::Error> {
        let mut heap = Heap::new();
        heap.add_reader("file1".to_string(), "a\nc".as_bytes())?;
        heap.add_reader("file2".to_string(), "b\nd".as_bytes())?;
        heap.add_reader("file3".to_string(), "".as_bytes())?;
        assert_eq!(heap.next().unwrap()?, "a");
        assert_eq!(heap.next().unwrap()?, "b");
        assert_eq!(heap.next().unwrap()?, "c");
        assert_eq!(heap.next().unwrap()?, "d");
        assert!(heap.next().is_none());
        Ok(())
    }

    #[test]
    fn test_multiple_with_dupes() -> Result<(), io::Error> {
        let mut heap = Heap::new();
        heap.add_reader("file1".to_string(), "a\nc".as_bytes())?;
        heap.add_reader("file2".to_string(), "b\nd".as_bytes())?;
        heap.add_reader("file3".to_string(), "b\nc".as_bytes())?;
        assert_eq!(heap.next().unwrap()?, "a");
        assert_eq!(heap.next().unwrap()?, "b");
        assert_eq!(heap.next().unwrap()?, "b");
        assert_eq!(heap.next().unwrap()?, "c");
        assert_eq!(heap.next().unwrap()?, "c");
        assert_eq!(heap.next().unwrap()?, "d");
        assert!(heap.next().is_none());
        Ok(())
    }

    #[test]
    fn test_multiple_with_repeated_names() -> Result<(), io::Error> {
        let mut heap = Heap::new();
        heap.add_reader("file1".to_string(), "a\nc".as_bytes())?;
        heap.add_reader("file1".to_string(), "b\nd".as_bytes())?;
        assert_eq!(heap.next().unwrap()?, "a");
        assert_eq!(heap.next().unwrap()?, "b");
        assert_eq!(heap.next().unwrap()?, "c");
        assert_eq!(heap.next().unwrap()?, "d");
        assert!(heap.next().is_none());
        Ok(())
    }

    #[test]
    fn test_multiple_empty() -> Result<(), io::Error> {
        let mut heap = Heap::new();
        heap.add_reader("file1".to_string(), "".as_bytes())?;
        heap.add_reader("file1".to_string(), "".as_bytes())?;
        assert!(heap.next().is_none());
        Ok(())
    }
}
