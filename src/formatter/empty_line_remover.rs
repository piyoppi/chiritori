use super::utils::line_break_pos_finder::find_next_line_break_pos;
use super::utils::line_break_pos_finder::find_prev_line_break_pos;
use super::Formatter;

pub struct EmptyLineRemover {}

impl Formatter for EmptyLineRemover {
    fn format(&self, content: &str, byte_pos: usize) -> (usize, usize) {
        let bytes = content.as_bytes();

        if !content.is_char_boundary(byte_pos) {
            panic!("Invalid byte position: {}", byte_pos);
        }

        if bytes.get(byte_pos) != Some(&b'\n') {
            return (byte_pos, byte_pos);
        }

        let is_next_line_empty = find_next_line_break_pos(content, bytes, byte_pos)
            .map(|pos| find_next_line_break_pos(content, bytes, pos + 1))
            .flatten()
            == None;
        let is_prev_line_empty = find_prev_line_break_pos(content, bytes, byte_pos)
            .map(|pos| find_prev_line_break_pos(content, bytes, pos))
            .flatten()
            == None;

        if is_next_line_empty && is_prev_line_empty {
            (byte_pos, byte_pos + 1)
        } else {
            (byte_pos, byte_pos)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let remover = EmptyLineRemover {};

        //                      10
        //             0123456789012345
        //             |        ^
        let content = "    hoge++  foo".replace('+', "\n");
        assert_eq!(remover.format(&content, 9), (9, 10));

        //                      10
        //             0123456789012345
        //             |         ^
        let content = "    hoge+++  foo".replace('+', "\n");
        assert_eq!(remover.format(&content, 10), (10, 10));

        //                      10
        //             0123456789012345
        //             |        ^
        let content = "    hoge+++  foo".replace('+', "\n");
        assert_eq!(remover.format(&content, 9), (9, 9));

        //                      10
        //             01234567890123456
        //             |         ^
        let content = "    hoge++ +  foo".replace('+', "\n");
        assert_eq!(remover.format(&content, 10), (10, 10));
    }
}
