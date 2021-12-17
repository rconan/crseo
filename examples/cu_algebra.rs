use crseo::{cu::CuType, cu::Single, Cu};

fn test_algebra() {
    let x: Cu<Single> = vec![1f32, 2f32, 3f32].into();
    let a_mat: Cu<Single> = vec![vec![1f32, 2f32]; 3].into();
    //    let mut y = a_mat.mv(&x);
    let mut y = (&a_mat) * (&x);
    println!("y: {:?}", Vec::<f32>::from(&mut y));
    let mut yscale = (&y) * 2f32;
    println!("yscale: {:?}", Vec::<f32>::from(&mut yscale));
    yscale -= (&a_mat) * (&x);
    println!("yscale: {:?}", Vec::<f32>::from(&mut yscale));
}
struct StateSpace<T: CuType> {
    a_mat: Cu<T>,
    b_mat: Cu<T>,
    c_mat: Cu<T>,
    d_mat: Option<Cu<T>>,
    x: Cu<T>,
    x_next: Cu<T>,
    pub y: Cu<T>,
}
impl StateSpace<Single> {
    pub fn new(
        a_mat: Vec<Vec<f32>>,
        b_mat: Vec<Vec<f32>>,
        c_mat: Vec<Vec<f32>>,
        d_mat: Option<Vec<Vec<f32>>>,
    ) -> Self {
        let n_x = a_mat[0].len();
        let n_y = c_mat[0].len();
        Self {
            a_mat: a_mat.into(),
            b_mat: b_mat.into(),
            c_mat: c_mat.into(),
            d_mat: d_mat.map(|x| x.into()),
            x: vec![0.0; n_x].into(),
            x_next: vec![0.0; n_x].into(),
            y: vec![0.0; n_y].into(),
        }
    }
    pub fn update(&mut self, u: &Cu<Single>) -> &mut Cu<Single> {
        self.y = &self.c_mat * &self.x;
        if let Some(ref d_mat) = self.d_mat {
            self.y = d_mat * u;
        }
        self.x_next = &self.a_mat * &self.x;
        self.x_next += &self.b_mat * u;
        self.x = self.x_next.clone();
        &mut self.y
    }
}

fn main() {
    test_algebra();

    let mut ss: StateSpace<Single> = StateSpace::new(
        vec![vec![1f32]],
        vec![vec![1f32]],
        vec![vec![-0.5f32]],
        None,
    );
    (0..50).for_each(|k| {
        let mut u = Cu::<Single>::from(vec![1f32]);
        u += ss.y.clone();
        println!("{:2}: {:.3}", k, Vec::<f32>::from(ss.update(&u))[0]);
    });

    /*
    let mut ss: StateSpace<Single> = StateSpace::new(
        vec![vec![0f32,1f32],vec![1f32,0f32]],
        vec![vec![1f32,0f32]],
        vec![vec![0f32],vec![-0.2f32]],
        None,
    );
    (0..50).for_each(|k| {
        let mut u = Cu::<Single>::from(vec![1f32]);
        u += ss.y.clone();
        println!("{:2}: {:.?}", k, Vec::<f32>::from(ss.update(&u)));
    });*/
}
