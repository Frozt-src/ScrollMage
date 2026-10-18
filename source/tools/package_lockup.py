"""Package unchanged native renders, and outline-only SVG masters for editors."""
from pathlib import Path
import hashlib,json,sys
import numpy as np
from PIL import Image
raw,output=map(Path,sys.argv[1:3]);output.mkdir(parents=True,exist_ok=True)
source=Path(__file__).resolve().parents[1]/'assets/brand/lockup-v2.paths.json'
geometry=json.loads(source.read_text());motion=json.loads((raw/'motion.json').read_text())
for name,size in [('lockup-transparent',(1320,440)),('lockup-sidebar',(216,72)),
    ('lockup-sidebar-168',(168,64)),('lockup-sidebar-160',(160,64)),('lockup-sidebar-128',(128,64))]:
    Image.frombytes('RGBA',size,(raw/(name+'.rgba')).read_bytes()).save(output/(name+'.png'))
frames=[]
with (raw/'lockup-frames.rgba').open('rb') as stream:
    for _ in range(80):frames.append(Image.frombytes('RGBA',(990,330),stream.read(990*330*4)).convert('RGB'))
reference=np.asarray(frames[0]); changes=[]
for frame in frames:
    difference=np.any(np.asarray(frame)!=reference,axis=2)
    y,x=np.nonzero(difference)
    if len(x):
        bounds=[int(x.min()),int(y.min()),int(x.max()+1),int(y.max()+1)];changes.append(bounds)
        assert bounds[0]>=877 and bounds[1]>=24 and bounds[2]<=902 and bounds[3]<=196,bounds
assert changes,'Wheel animation must move'
palette=frames[0].quantize(colors=256,method=Image.Quantize.MEDIANCUT)
indexed=[frame.quantize(palette=palette,dither=Image.Dither.NONE) for frame in frames]
indexed[0].save(output/'lockup-elastic.gif',save_all=True,append_images=indexed[1:],duration=50,
    loop=0,optimize=False,disposal=1)
# Contact sheet uses actual native frames, never an approximation of the motion.
samples=[0,11,16,40,51,56]
sheet=Image.new('RGB',(990,660),(237,239,245))
for index,frame in enumerate(samples):
    thumb=frames[frame].resize((495,165),Image.Resampling.LANCZOS)
    sheet.paste(thumb,((index%2)*495,(index//2)*220+25))
sheet.save(output/'lockup-motion-sheet.png')
def contour(points):return 'M'+' L'.join(f'{x:g},{y:g}' for x,y in points)+' Z'
def capsule(p):
    x,y,w,h=[p[k] for k in ('x','y','width','height')];r=w/2;cx=x+r
    return f'M {cx:g},{y:g} A {r:g},{r:g} 0 0 1 {x+w:g},{y+r:g} V {y+h-r:g} A {r:g},{r:g} 0 0 1 {cx:g},{y+h:g} A {r:g},{r:g} 0 0 1 {x:g},{y+h-r:g} V {y+r:g} A {r:g},{r:g} 0 0 1 {cx:g},{y:g} Z'
ink=' '.join(map(contour,geometry['ink']))+' '+capsule(geometry['stem']);mage=' '.join(map(contour,geometry['mage']))
start='<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1320 440" width="1320" height="440" role="img" aria-labelledby="title"><title id="title">Scroll Mage</title>'
static=start+f'<path id="mouse-and-scroll" fill="#000000" fill-rule="evenodd" d="{ink} {capsule(motion["rest"])}"/><path id="mage" fill="{geometry["purple"]}" fill-rule="evenodd" d="{mage}"/></svg>'
(output/'ScrollMage-Master.svg').write_text(static)
animated=start+'<style>.still{display:none}@media(prefers-reduced-motion:reduce){.motion{display:none}.still{display:inline}}</style><defs><mask id="wheel-cutout" maskUnits="userSpaceOnUse" x="0" y="0" width="1320" height="440"><rect width="1320" height="440" fill="white"/>'
animated+=f'<path class="still" fill="black" d="{capsule(motion["rest"])}"/>'
values=';'.join(capsule(p) for p in motion['frames'])
times=';'.join(f'{i/80:g}' for i in range(81))
animated+=f'<path class="motion" fill="black" d="{capsule(motion["frames"][0])}"><animate attributeName="d" values="{values}" keyTimes="{times}" dur="4s" repeatCount="indefinite" calcMode="linear"/></path></mask></defs>'
animated+=f'<path fill="#000000" fill-rule="evenodd" mask="url(#wheel-cutout)" d="{ink}"/><path fill="{geometry["purple"]}" fill-rule="evenodd" d="{mage}"/></svg>'
(output/'ScrollMage-Animated.svg').write_text(animated)
logo=Image.open(output/'lockup-transparent.png');assert logo.getpixel((0,0))[3]==0
assert logo.getpixel((1186,150))[3]==0
assert logo.getpixel((1186,240))[3]==255
animation=Image.open(output/'lockup-elastic.gif');duration=0
for i in range(animation.n_frames):animation.seek(i);duration+=animation.info['duration']
assert duration==4000
report={'design':'user-supplied mouse tail connected to S, purple MAGE under the right',
    'renderer':'same cached native vector Lockup used by the app','dimensions':[1320,440],
    'transparent_static_master':True,'embedded_raster_images':False,'font_dependencies':False,
    'wheel_cutout_rebuilt':True,'animation_changes_only_final_l':True,'source_frames':80,
    'encoded_frames':animation.n_frames,'duration_ms':duration,'source_sha256':geometry['source_sha256'],
    'svg_master':'ScrollMage-Master.svg','animated_svg':'ScrollMage-Animated.svg'}
(output/'lockup-validation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))

