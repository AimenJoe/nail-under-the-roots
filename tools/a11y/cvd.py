"""색약 점검(0150): 스크린숏 한 칸을 원본 · 1형 · 2형 · 3형 · 흑백 다섯 칸으로 나란히.

행렬: Machado, Oliveira & Fernandes 2009, 정도 1.0(colour-science 0.4.7 datasets/machado2010.py에서 옮김).
선형 RGB에서 곱한다. 흑백은 Rec.709 휘도. 알아야 하는 것(위험·거절·예고)은 다섯 칸 모두에서 남아야 한다(KB guides/color-vision-check-for-games.md).

사용: python3 tools/a11y/cvd.py 입력.png x0,y0,x1,y1 출력.png [배율]"""
import sys
import numpy as np
from PIL import Image, ImageDraw
M = {
 'protan': np.array([[0.152286, 1.052583, -0.204868],[0.114503, 0.786281, 0.099216],[-0.003882, -0.048116, 1.051998]]),
 'deutan': np.array([[0.367322, 0.860646, -0.227968],[0.280085, 0.672501, 0.047413],[-0.011820, 0.042940, 0.968881]]),
 'tritan': np.array([[1.255528, -0.076749, -0.178779],[-0.078411, 0.930809, 0.147602],[0.004733, 0.691367, 0.303900]]),
}
def lin(c): return np.where(c <= 0.04045, c/12.92, ((c+0.055)/1.055)**2.4)
def srgb(c): c=np.clip(c,0,1); return np.where(c <= 0.0031308, c*12.92, 1.055*c**(1/2.4)-0.055)
def sim(img, m):
    a = lin(np.asarray(img.convert('RGB'), dtype=np.float64)/255.0)
    b = a @ m.T
    return Image.fromarray((srgb(b)*255+0.5).astype(np.uint8))
def gray(img):
    a = lin(np.asarray(img.convert('RGB'), dtype=np.float64)/255.0)
    y = a @ np.array([0.2126, 0.7152, 0.0722])
    v = (srgb(y)*255+0.5).astype(np.uint8)
    return Image.fromarray(np.stack([v]*3, -1))
src, box, out = sys.argv[1], tuple(int(v) for v in sys.argv[2].split(',')), sys.argv[3]
k = int(sys.argv[4]) if len(sys.argv) > 4 else 2
im = Image.open(src).convert('RGB').crop(box)
views = [('원본', im)] + [(n, sim(im, M[n])) for n in ['protan','deutan','tritan']] + [('흑백', gray(im))]
w, h = im.size[0]*k, im.size[1]*k
sheet = Image.new('RGB', (w*len(views), h+14), (20,20,20))
d = ImageDraw.Draw(sheet)
for i,(n,v) in enumerate(views):
    sheet.paste(v.resize((w,h), Image.NEAREST), (i*w, 14))
    d.text((i*w+4, 1), {'원본': 'orig', '흑백': 'gray'}.get(n, n), fill=(255,255,255))
sheet.save(out)
