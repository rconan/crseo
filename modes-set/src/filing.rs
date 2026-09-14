use interface::filing::Codec;

impl Codec for crate::Set {}
impl<M: crate::Mesh> Codec for crate::ModesSets<M> {}
