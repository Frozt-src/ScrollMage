"""Package native-rendered logo frames and a small TrueType lettering font. No font install."""
import json, math, struct, sys, time
from pathlib import Path
from PIL import Image, ImageFont, features

source, output = map(Path, sys.argv[1:3])
output.mkdir(parents=True, exist_ok=True)
data = json.loads((source / 'outlines.json').read_text())
samples = json.loads((source / 'animation.json').read_text())

def svg_path(contours, offset=0):
    return ' '.join('M'+' L'.join(f'{x+offset:.4f},{y:.4f}' for x,y in c)+' Z' for c in contours)

word = ' '.join(svg_path(data['glyphs'][i]['contours'], x) for i,x in data['word'])
rest = data['pill']
def pill_rect(p, extra=''):
    return f'<rect x="{p["x"]:.4f}" y="{p["y"]:.4f}" width="{p["width"]:.4f}" height="{p["height"]:.4f}" rx="{p["width"]/2:.4f}" fill="black" {extra}/>'

def svg(animated):
    if animated:
        attrs = ''.join(f'<animate attributeName="{key}" values="'+ ';'.join(f'{(p["width"]/2 if key=="rx" else p[key]):.4f}' for p in samples) +'" dur="4s" repeatCount="indefinite" calcMode="linear"/>' for key in ['x','y','width','height','rx'])
        mask = f'<g class="motion">{pill_rect(samples[0])[:-2]}>{attrs}</rect></g><g class="still">{pill_rect(rest)}</g>'
        style = '<style>.still{display:none}@media(prefers-reduced-motion:reduce){.motion{display:none}.still{display:inline}}</style>'
    else:
        mask,style = pill_rect(rest),''
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 140" role="img" aria-labelledby="title"><title id="title">Scroll Mage scroll lettering</title>{style}<defs><mask id="wheel" maskUnits="userSpaceOnUse" x="0" y="0" width="460" height="140"><rect width="460" height="140" fill="white"/>{mask}</mask></defs><path d="{word}" fill="black" fill-rule="evenodd" mask="url(#wheel)"/></svg>'

(output / 'scroll-static.svg').write_text(svg(False), encoding='utf-8')
(output / 'scroll-animated.svg').write_text(svg(True), encoding='utf-8')
Image.frombytes('RGBA',(672,402),(source/'static.rgba').read_bytes()).save(output/'scroll-reference.png')
Image.frombytes('RGBA',(460,140),(source/'transparent.rgba').read_bytes()).save(output/'scroll-transparent.png')
frames=[]
with (source/'frames.rgba').open('rb') as stream:
    for _ in range(200):
        frames.append(Image.frombytes('RGBA',(672,402),stream.read(672*402*4)).convert('RGB'))
frames[0].save(output/'scroll-elastic.gif',save_all=True,append_images=frames[1:],duration=20,loop=0,optimize=True,disposal=1)

def pack(fmt,*values): return struct.pack('>'+fmt,*values)
def inside(p,c):
    x,y=p;result=False
    for a,b in zip(c,c[1:]+c[:1]):
        if (a[1]>y)!=(b[1]>y) and x < (b[0]-a[0])*(y-a[1])/(b[1]-a[1])+a[0]: result=not result
    return result
def normalized(contours):
    contours=[c[:-1] if c[0]==c[-1] else c[:] for c in contours if len(c)>=3]
    result=[]
    for i,c in enumerate(contours):
        depth=sum(inside(c[0],other) for j,other in enumerate(contours) if j!=i)
        points=[(round(x*6),round((137-y)*6)) for x,y in c]
        points=[p for j,p in enumerate(points) if j==0 or p!=points[j-1]]
        if points[0]==points[-1]:points.pop()
        area=sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(points,points[1:]+points[:1]))
        if (area<0)!=(depth%2==0): points.reverse()
        result.append(points)
    return result
def rounded_rect(x,y,w,h):
    radius=w/2
    return [[x+radius+radius*math.cos(a),cy+radius*math.sin(a)] for cy,start in [(y+radius,math.pi),(y+h-radius,0)] for a in [start+j*math.pi/32 for j in range(33)]]

glyphs=[{'name':'.notdef','advance':500,'contours':[[(40,0),(40,700),(460,700),(460,0)]]},{'name':'space','advance':300,'contours':[]}]
for g in data['glyphs']:
    glyphs.append({'name':g['name'],'advance':round(g['advance']*6),'contours':normalized(g['contours'])})
glyph_index={g['name']:i for i,g in enumerate(glyphs)}
l = data['glyphs'][4]['contours']
ll = [c[:] for c in l]+[[[x+54,y] for x,y in c] for c in l]+[rounded_rect(57,41.5,17,54)]
glyphs.append({'name':'l_l','advance':468,'contours':normalized(ll)})

glyf=bytearray();loca=[];metrics=bytearray();bounds=[];max_points=max_contours=0
for glyph in glyphs:
    loca.append(len(glyf));cs=glyph['contours'];points=[p for c in cs for p in c]
    max_points=max(max_points,len(points));max_contours=max(max_contours,len(cs))
    bbox=(min(p[0] for p in points),min(p[1] for p in points),max(p[0] for p in points),max(p[1] for p in points)) if points else (0,0,0,0)
    bounds.append(bbox);metrics+=pack('Hh',glyph['advance'],bbox[0])
    if not points:continue
    glyf+=pack('hhhhh',len(cs),*bbox)
    end=-1
    for c in cs:end+=len(c);glyf+=pack('H',end)
    glyf+=pack('H',0)+bytes([1])*len(points)
    for axis in [0,1]:
        previous=0
        for p in points:glyf+=pack('h',p[axis]-previous);previous=p[axis]
    glyf+=bytes((-len(glyf))%4)
loca.append(len(glyf))
bbox=(min(b[0] for b in bounds),min(b[1] for b in bounds),max(b[2] for b in bounds),max(b[3] for b in bounds))
timestamp=int(time.time())+2082844800
head=pack('IIIIHHQQhhhhHHhhh',0x10000,round(0.4*65536),0,0x5f0f3cf5,11,1000,timestamp,timestamp,*bbox,0,8,2,1,0)
hhea=pack('IhhhH'+'h'*11+'H',0x10000,900,-100,0,max(g['advance'] for g in glyphs),min(b[0] for b in bounds),min(g['advance']-b[2] for g,b in zip(glyphs,bounds)),max(b[2] for b in bounds),1,0,0,0,0,0,0,0,len(glyphs))
maxp=pack('I'+'H'*14,0x10000,len(glyphs),max_points,max_contours,0,0,1,0,0,0,0,0,0,0,0)
codes=[(32,1)]+sorted((ord(g['character']),glyph_index[g['name']]) for g in data['glyphs'])+[(0xffff,0)]
n=len(codes);power=2**int(math.log2(n))
cmap4=pack('HHHHHHH',4,16+8*n,0,2*n,2*power,int(math.log2(n)),2*n-2*power)+b''.join(pack('H',c) for c,_ in codes)+pack('H',0)+b''.join(pack('H',c) for c,_ in codes)+b''.join(pack('H',(gid-c)&65535) for c,gid in codes)+bytes(2*n)
cmap=pack('HHHHIHHI',0,2,0,3,20,3,1,20)+cmap4
names={1:'Scroll Mage Scroll',2:'Regular',3:'ScrollMageScroll-Regular-0.4',4:'Scroll Mage Scroll Lettering',5:'Version 0.400',6:'ScrollMageScroll-Regular',16:'Scroll Mage Scroll',17:'Regular'}
name_records=bytearray();name_strings=bytearray()
for key,value in names.items():
    encoded=value.encode('utf-16-be');name_records+=pack('HHHHHH',3,1,0x409,key,len(encoded),len(name_strings));name_strings+=encoded
name=pack('HHH',0,len(names),6+12*len(names))+name_records+name_strings
os2=pack('HhHHH',4,500,400,5,0)+pack('h'*11,650,600,0,75,650,600,0,350,50,300,0)+bytes(10)+pack('IIII',1,0,0,0)+b'SMGE'+pack('HHHhhhHHIIhhHHH',0x40,32,115,900,-100,0,900,100,1,0,600,822,0,32,2)
assert len(os2)==96 and len(head)==54 and len(hhea)==36 and len(maxp)==32
lang=pack('HHHH',0,0xffff,1,0);script=pack('HH',4,0)+lang
scriptlist=pack('H',2)+b'DFLT'+pack('H',14)+b'latn'+pack('H',26)+script+script
featurelist=pack('H',1)+b'liga'+pack('H',8)+pack('HHH',0,1,0)
subtable=pack('HHHH',1,18,1,8)+pack('HH',1,4)+pack('HHH',len(glyphs)-1,2,glyph_index['l'])+pack('HHH',1,1,glyph_index['l'])
lookuplist=pack('HHHHHH',1,4,4,0,1,8)+subtable
gsub=pack('IHHH',0x10000,10,10+len(scriptlist),10+len(scriptlist)+len(featurelist))+scriptlist+featurelist+lookuplist
tables={'head':head,'hhea':hhea,'maxp':maxp,'hmtx':metrics,'glyf':glyf,'loca':b''.join(pack('I',o) for o in loca),'cmap':cmap,'name':name,'OS/2':os2,'post':pack('IihhIIIII',0x30000,0,-100,50,0,0,0,0,0),'GSUB':gsub}
def checksum(value):
    value=bytes(value)+bytes((-len(value))%4)
    return sum(struct.unpack('>'+'I'*(len(value)//4),value))&0xffffffff
count=len(tables);power=2**int(math.log2(count));font=bytearray(pack('IHHHH',0x10000,count,power*16,int(math.log2(count)),count*16-power*16));body=bytearray();records=bytearray();offset=12+16*count;head_offset=0
for tag,table in sorted(tables.items()):
    records+=tag.encode('ascii')+pack('III',checksum(table),offset,len(table))
    if tag=='head':head_offset=offset
    body+=table+bytes((-len(table))%4);offset+=len(table)+(-len(table))%4
font+=records+body;font[head_offset+8:head_offset+12]=pack('I',(0xb1b0afba-checksum(font))&0xffffffff)
assert checksum(font)==0xb1b0afba
font_path=output/'ScrollMage-Scroll.ttf';font_path.write_bytes(font)
loaded=ImageFont.truetype(str(font_path),120)
assert loaded.getmask('scroll').getbbox() is not None
gif=Image.open(output/'scroll-elastic.gif')
duration=0
for index in range(gif.n_frames):
    gif.seek(index);duration+=gif.info.get('duration',0)
assert duration==4000
transparent=Image.open(output/'scroll-transparent.png')
assert transparent.getpixel((447,68))[3]==0 and transparent.getpixel((393,68))[3]==255
import xml.etree.ElementTree as ET
ET.parse(output/'scroll-animated.svg');ET.parse(output/'scroll-static.svg')
report={'font_bytes':len(font),'glyph_count':len(glyphs),'characters':' clors','full_alphabet':False,'checksum':'0xb1b0afba','freetype_load':'passed','raqm_available':features.check('raqm'),'source_frames':200,'encoded_gif_frames':gif.n_frames,'gif_duration_ms':duration,'transparent_counter':'passed','svg_xml':'passed','source':'same native vector outlines and motion samples used by the app'}
if features.check('raqm'):
    enabled=loaded.getlength('ll',features=['liga']);disabled=loaded.getlength('ll',features=['-liga'])
    assert enabled<disabled
    report['ll_ligature']='passed';report['ll_advance_with_ligature']=enabled;report['ll_advance_without_ligature']=disabled
(output/'asset-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
