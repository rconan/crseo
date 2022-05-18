# CRSEO: [Cuda Engined Optics](https://github.com/rconan/ceo) Rust Wrapper

## Installation

1. CEO install

Install [CUDA](https://developer.nvidia.com/cuda-10.2-download-archive) and [Noweb](https://www.cs.tufts.edu/~nr/noweb/), then
```
git clone -b rust https://github.com/rconan/ceo.git
cd CEO
make all
sudo make install
cd ..
```
2. GMT M1 and M2 modes
```
mkdir data
cd data
wget https://s3.us-west-2.amazonaws.com/gmto.modeling/ceo-modes.tar
tar xvf ceo-modes.tar
export GMT_MODES_PATH=`pwd`
cd ..
```
