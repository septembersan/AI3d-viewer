"""Point Cloud Viewer サンプル集"""

import numpy as np
import point_cloud_viewer as pcv


def random_sphere(n=50_000):
    """球面上のランダムポイント"""
    phi = np.random.uniform(0, 2 * np.pi, n)
    cos_theta = np.random.uniform(-1, 1, n)
    sin_theta = np.sqrt(1 - cos_theta**2)

    positions = np.column_stack([
        sin_theta * np.cos(phi),
        sin_theta * np.sin(phi),
        cos_theta,
    ]).astype(np.float32)

    # 高度で青→赤グラデーション
    t = (positions[:, 2] + 1) / 2  # 0..1
    colors = np.column_stack([
        (t * 255),
        (50 * np.ones(n)),
        ((1 - t) * 255),
    ]).astype(np.uint8)

    pcv.show(positions, colors)


def gaussian_blob(n=100_000):
    """3Dガウス分布（位置ベース自動着色）"""
    positions = np.random.randn(n, 3).astype(np.float32)
    pcv.show(positions)


def helix(n=30_000):
    """二重らせん"""
    t = np.linspace(0, 8 * np.pi, n // 2)

    x1 = np.cos(t)
    y1 = np.sin(t)
    z1 = t / (8 * np.pi) * 4 - 2

    x2 = np.cos(t + np.pi)
    y2 = np.sin(t + np.pi)
    z2 = z1.copy()

    positions = np.row_stack([
        np.column_stack([x1, y1, z1]),
        np.column_stack([x2, y2, z2]),
    ]).astype(np.float32)

    colors = np.row_stack([
        np.tile([255, 80, 80], (len(t), 1)),
        np.tile([80, 80, 255], (len(t), 1)),
    ]).astype(np.uint8)

    pcv.show(positions, colors)


def torus(n=80_000):
    """トーラス（ドーナツ形状）"""
    R, r = 2.0, 0.6
    theta = np.random.uniform(0, 2 * np.pi, n)
    phi = np.random.uniform(0, 2 * np.pi, n)

    positions = np.column_stack([
        (R + r * np.cos(phi)) * np.cos(theta),
        (R + r * np.cos(phi)) * np.sin(theta),
        r * np.sin(phi),
    ]).astype(np.float32)

    # theta, phi で色分け
    colors = np.column_stack([
        ((np.sin(theta) + 1) / 2 * 255),
        ((np.cos(phi) + 1) / 2 * 255),
        ((np.sin(phi) + 1) / 2 * 200 + 55),
    ]).astype(np.uint8)

    pcv.show(positions, colors)


SAMPLES = {
    "sphere": ("球面", random_sphere),
    "gaussian": ("ガウス分布", gaussian_blob),
    "helix": ("二重らせん", helix),
    "torus": ("トーラス", torus),
    "demo": ("デモキューブ", pcv.show_demo),
}

if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2 or sys.argv[1] not in SAMPLES:
        print("使い方: python sample.py <サンプル名>\n")
        for key, (desc, _) in SAMPLES.items():
            print(f"  {key:10s}  {desc}")
        sys.exit(1)

    name = sys.argv[1]
    desc, fn = SAMPLES[name]
    print(f"{desc} を表示します...")
    fn()
