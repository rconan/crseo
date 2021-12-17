import numpy as np
import os

rust = dict(np.load("windloading.20210225_1447_MT_mount_v202102_ASM_wind2.pkl",allow_pickle=True))
m1_rbm = np.asarray([x[1] for x in rust['OSSM1Lcl']['data'] ]) 
m2_rbm = np.asarray([x[1] for x in rust['MCM2RB6D']['data'] ])
rbm = np.hstack([m1_rbm,m2_rbm])
D_jitter = np.load(os.path.join("/home/rconan/Documents/GMT/CFD/Python/CFD/",'linear_jitter.npz'))
jitter = D_jitter['D_tt']@rbm.T * 180*3600e3/np.pi
print(np.std(jitter[:,3000:],1) )
