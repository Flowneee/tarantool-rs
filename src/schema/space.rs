use crate::ConnectionLike;

pub struct Space {
    conn: Box<dyn ConnectionLike>,
}
