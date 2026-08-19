# 行为学对照集

每种数字生物对照**最接近的真实类群**，指标独立，训练时只抖动该种参数。对照表见 `taxa.json`。

灯鱼学校只作细长鱼群的文献锚，**不套全场**。全场 17 种里，只有磷虾按强集群打分。

## 分种训练

观察循环 `observe_record_optimize_loop` 按 `cycle % 17` 轮转物种：

1. 开 N=8 的同种学校，用该种的 NND / 转向 / 极化 / 游速打分
2. `mutate_species` 只改该种生活参数（含 `pace`）；辐射种不抖航向
3. 接受变好时只写回 `LIFE.kinds[ci]`，全局场（body/near/far…）不动

## 灯鱼（仅文献）

灯鱼 *Hemigrammus rhodostomus* 群体轨迹，用来把仿真指标换成**体长**并和真实分布比。

| | |
|---|---|
| 来源 | Puy et al. 2024, PNAS [10.1073/pnas.2309733121](https://doi.org/10.1073/pnas.2309733121) |
| 数据 | Zenodo [10.5281/zenodo.10890112](https://zenodo.org/records/10890112) |
| 标定 | 50 fps；2745 px = 100 cm；体长按 3.5 cm |
| 本地 | 两场 N=8、各约 30 min（`Experimental_school_N=8_recording_{0,1}.csv`，已 gitignore） |

生成对照：

```
python3 scripts/ethology/build_ref.py
```

写出 `ref.json`。汇总值（两场平均）：

- 最近邻距中位 **0.99 BL**（p10 0.56，p90 1.60）
- 极化中位 **0.98**
- 转向率中位 **0.51 rad/s**（p90 2.44）
- 路径急转（0.2 s 内 >75°）**0.93%**
- 活跃游速中位 **4.4 BL/s**（已丢掉 <1 BL/s 的静止段）

## 数字种 ← 最近真实种

| ci | id | 对照 | nnd | yaw | polar | speed BL/s |
|---|---|---|---|---|---|---|
| 0 | fucan | Tomopteris 浮蚕 | 2.80 | 0.14 | 0.32 | 1.20 |
| 1 | youyan | 端足/海蟑螂类划水 | 2.00 | 0.12 | 0.50 | 1.50 |
| 2 | jichong | Nereis 游走多毛类 | 3.00 | 0.18 | 0.28 | 1.30 |
| 3 | jelly | Aurelia aurita | 2.20 | 0.22 | 0.38 | 0.35 |
| 4 | nebula | Cyanea/Chrysaora | 2.50 | 0.20 | 0.32 | 0.28 |
| 5 | lantern | Aequorea 花水母 | 2.00 | 0.25 | 0.40 | 0.45 |
| 6 | feather | 缨鳃虫 | 2.40 | 0.06 | 0 | 0.12 |
| 7 | tentacle | 浮蚕/须虫 | 2.80 | 0.16 | 0.26 | 1.10 |
| 8 | flower6 | 银币/僧帽漂浮体 | 3.00 | 0 | 0 | 0.12 |
| 9 | wheel | 轮虫 Brachionus | 2.50 | 0 | 0 | 0.18 |
| 10 | spiral | 螺蛸/Limacina | 3.20 | 0.20 | 0.30 | 0.60 |
| 11 | comb | Mnemiopsis | 3.20 | 0.18 | 0.22 | 0.12 |
| 12 | saweel | Anguilla | 3.00 | 0.14 | 0.40 | 1.40 |
| 13 | star8 | 海星/八放珊瑚 | 2.80 | 0 | 0 | 0.10 |
| 14 | shrimp | Euphausia superba | 1.05 | 0.10 | 0.78 | 2.00 |
| 15 | vortex | 涡虫 | 2.60 | 0.15 | 0.25 | 0.80 |
| 16 | angel | Clione limacina | 3.00 | 0.12 | 0.28 | 0.70 |

辐射种（花、轮、星）`w_yaw=0, w_polar=0`，不逼极化。磷虾才是密集群。
