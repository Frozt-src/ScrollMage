"""Render the existing editable mouse paths into squircle SVG, PNG and ICO assets."""
from pathlib import Path
import hashlib, json, math, re, struct, xml.etree.ElementTree as ET
import numpy as np
from PIL import Image, ImageDraw, ImageChops

OUT = Path(__file__).resolve().parents[1] / 'assets/icons'
SOURCE = OUT / 'mouse-source.svg'
PATHS = [(p.attrib.get('id', ''), p.attrib['d'])
         for p in ET.fromstring(SOURCE.read_bytes()).iter('{http://www.w3.org/2000/svg}path')]

def flatten(data):
    tokens = re.findall(r'[MmLlCcSsZz]|[-+]?(?:\d*\.\d+|\d+\.?)(?:[eE][-+]?\d+)?', data)
    i, command, point, start, previous = 0, '', (0., 0.), (0., 0.), None
    contours, contour = [], []
    while i < len(tokens):
        if tokens[i].isalpha():
            command = tokens[i]; i += 1
            if command.upper() == 'Z':
                contours.append(contour); contour = []; point = start; previous = None
                continue
        code = command.upper(); relative = command.islower()
        count = {'M':2, 'L':2, 'C':6, 'S':4}[code]
        values = list(map(float, tokens[i:i+count])); i += count
        pairs = [(values[j] + (point[0] if relative else 0), values[j+1] + (point[1] if relative else 0))
                 for j in range(0, count, 2)]
        if code in ('M', 'L'):
            point = pairs[0]; previous = None
            if code == 'M':
                if contour: contours.append(contour)
                contour = []; start = point; command = 'l' if relative else 'L'
            contour.append(point)
        else:
            a = pairs[0] if code == 'C' else (2*point[0]-previous[0], 2*point[1]-previous[1]) if previous else point
            b, end = pairs[-2:]
            for step in range(1, 41):
                t = step / 40; u = 1-t
                contour.append(tuple(u*u*u*point[k]+3*u*u*t*a[k]+3*u*t*t*b[k]+t*t*t*end[k] for k in (0, 1)))
            point, previous = end, b
    if contour: contours.append(contour)
    return contours

SHAPES = [(name, data, flatten(data)) for name, data in PATHS]
POINTS = [p for _, _, contours in SHAPES for c in contours for p in c]
MINX, MAXX = min(p[0] for p in POINTS), max(p[0] for p in POINTS)
MINY, MAXY = min(p[1] for p in POINTS), max(p[1] for p in POINTS)

def squircle(inset):
    radius = 256-inset
    return [(256 + radius*math.copysign(abs(math.cos(a))**(2/4.6), math.cos(a)),
             256 + radius*math.copysign(abs(math.sin(a))**(2/4.6), math.sin(a)))
            for a in (i*math.tau/192 for i in range(192))]

def transform(small):
    width = 442 if small else 414
    scale = width / (MAXX-MINX)
    return scale, (512-width)/2 + MAXX*scale, 252-(MAXY-MINY)*scale/2-MINY*scale

def svg(small=False):
    shape = 'M'+' L'.join(f'{x:.3f},{y:.3f}' for x,y in squircle(19))+' Z'
    scale, x, y = transform(small)
    paths = ''.join(f'<path fill="#FFFFFF" fill-rule="evenodd" d="{data}"/>' for name,data,_ in SHAPES
                    if not small or 'foot' not in name.lower())
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" role="img" aria-labelledby="title">
<title id="title">Scroll Mage gunmetal mouse icon</title>
<defs><linearGradient id="gunmetal" x2="0" y2="1"><stop stop-color="#363C45"/><stop offset="1" stop-color="#111418"/></linearGradient>
<linearGradient id="rim" x2="0" y2="1"><stop stop-color="#707986"/><stop offset="1" stop-color="#080A0D"/></linearGradient></defs>
<path d="{shape}" fill="url(#gunmetal)" stroke="url(#rim)" stroke-width="1.6"/>
<g transform="matrix({-scale:.8f} 0 0 {scale:.8f} {x:.8f} {y:.8f})">{paths}</g></svg>'''

def render(size, small=False):
    ss = 2 if size >= 512 else 4
    n = size*ss; factor = n/512
    mask = Image.new('L', (n,n)); ImageDraw.Draw(mask).polygon([(x*factor,y*factor) for x,y in squircle(19)], fill=255)
    face = np.zeros((n,n,4), dtype=np.uint8)
    t = np.linspace(0,1,n,dtype=np.float32)[:,None]
    for k,(a,b) in enumerate(zip((54,60,69),(17,20,24))): face[:,:,k] = (a+(b-a)*t).astype(np.uint8)
    face[:,:,3] = np.asarray(mask)
    image = Image.fromarray(face)
    edge = Image.new('RGBA',(n,n)); d = ImageDraw.Draw(edge)
    d.line([(x*factor,y*factor) for x,y in squircle(19)]+[((256+237)*factor,256*factor)], fill=(118,128,143,120), width=max(1,round(1.1*factor)))
    edge.putalpha(ImageChops.multiply(edge.getchannel('A'),mask)); image.alpha_composite(edge)
    mark = Image.new('L',(n,n)); scale,x,y = transform(small)
    for name,_,contours in SHAPES:
        if small and 'foot' in name.lower(): continue
        layer=Image.new('L',(n,n)); draw=ImageDraw.Draw(layer)
        for index,contour in enumerate(contours):
            draw.polygon([((x-px*scale)*factor,(y+py*scale)*factor) for px,py in contour],fill=255 if index==0 else 0)
        mark=ImageChops.lighter(mark,layer)
    white=Image.new('RGBA',(n,n),'white');white.putalpha(mark);image.alpha_composite(white)
    image = image.resize((size,size),Image.Resampling.LANCZOS)
    pixels = np.asarray(image).copy()
    pixels[pixels[:,:,3] < 3] = 0
    return Image.fromarray(pixels)

def ico(path, sizes):
    entries=[];chunks=[];offset=6+16*len(sizes)
    for size in sizes:
        image=render(size,size<=32)
        if size==256:
            import io
            stream=io.BytesIO();image.save(stream,format='PNG');data=stream.getvalue()
        else:
            rows=np.asarray(image)[::-1]
            bgra=rows[:,:,[2,1,0,3]].tobytes()
            stride=((size+31)//32)*4;and_mask=bytearray(stride*size)
            for y in range(size):
                for x in range(size):
                    if rows[y,x,3]==0:and_mask[y*stride+x//8]|=128>>(x%8)
            data=struct.pack('<IiiHHIIiiII',40,size,size*2,1,32,0,len(bgra),0,0,0,0)+bgra+and_mask
        entries.append(struct.pack('<BBBBHHII',size%256,size%256,0,0,1,32,len(data),offset));chunks.append(data);offset+=len(data)
    path.write_bytes(struct.pack('<HHH',0,1,len(sizes))+b''.join(entries)+b''.join(chunks))

def main():
    OUT.mkdir(parents=True,exist_ok=True)
    (OUT/'ScrollMage-Icon.svg').write_text(svg())
    (OUT/'ScrollMage-Tray.svg').write_text(svg(True))
    render(1024).save(OUT/'ScrollMage-Icon.png')
    sizes=[16,20,24,28,32,40,48,64,96,128,256]
    for size in sizes:render(size,size<=32).save(OUT/f'icon-{size}.png')
    ico(OUT/'ScrollMage.ico',sizes)
    ico(OUT/'ScrollMage-tray.ico',sizes[:8])
    report={'source_sha256':hashlib.sha256(SOURCE.read_bytes()).hexdigest(),'source_paths':len(PATHS),
            'style':'Gunmetal continuous squircle, white existing mouse, left-facing to match the title logo',
            'sizes':sizes,'tray_sizes':sizes[:8],'small_cut':'Larger mark, far feet omitted for clarity',
            'transparent_corners':True,'new_third_party_dependencies':False}
    (OUT/'icon-validation.json').write_text(json.dumps(report,indent=2))
    print(json.dumps(report,indent=2))

if __name__=='__main__':main()
