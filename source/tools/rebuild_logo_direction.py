"""Reconstruct supplied artwork as outline data for the native logo renderer.
Authoring only: Pillow and numpy are not app runtime dependencies.
"""
from pathlib import Path
import hashlib,json,math
from collections import defaultdict
import numpy as np
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'assets/brand/logo-direction-v2.png'
a=np.asarray(Image.open(SOURCE).convert('RGB'))
ink=a.max(axis=2)<110
ink[:,1340:]=False # Rebuild the final l as a solid stem, with a live wheel cutout.
purple=(a[:,:,2]>125)&(a[:,:,1]<120)&(a[:,:,0]>65)&(a[:,:,0]<210)
ORIGIN=(190,270)
def simplify(points,epsilon=1.2):
    if len(points)<=2:return points
    start=np.array(points[0],dtype=float); end=np.array(points[-1],dtype=float)
    array=np.array(points,dtype=float);line=end-start;length=float(line@line)
    if length:
        t=np.clip((array-start)@line/length,0,1)
        distances=np.linalg.norm(array-(start+t[:,None]*line),axis=1)
    else:distances=np.linalg.norm(array-start,axis=1)
    index=int(distances.argmax())
    if distances[index]<=epsilon:return [points[0],points[-1]]
    return simplify(points[:index+1],epsilon)[:-1]+simplify(points[index:],epsilon)
def contours(mask):
    padded=np.pad(mask,1)
    neighbors=[padded[:-2,1:-1],padded[1:-1,2:],padded[2:,1:-1],padded[1:-1,:-2]]
    edges=defaultdict(list)
    for side,neighbor in enumerate(neighbors):
        ys,xs=np.nonzero(mask&~neighbor)
        for y,x in zip(ys.tolist(),xs.tolist()):
            if side==0:start,end=(x,y),(x+1,y)
            elif side==1:start,end=(x+1,y),(x+1,y+1)
            elif side==2:start,end=(x+1,y+1),(x,y+1)
            else:start,end=(x,y+1),(x,y)
            edges[start].append(end)
    result=[]
    def direction(p,q):return {(1,0):0,(0,1):1,(-1,0):2,(0,-1):3}[(q[0]-p[0],q[1]-p[1])]
    while edges:
        first=next(iter(edges)); point=first;previous=None;path=[]
        while True:
            path.append(point);options=edges[point]
            if previous is None:following=options[0]
            else:
                incoming=direction(previous,point)
                following=min(options,key=lambda nxt:{1:0,0:1,3:2,2:3}[(direction(point,nxt)-incoming)%4])
            options.remove(following)
            if not options:del edges[point]
            previous,point=point,following
            if point==first:break
        area=sum(p[0]*q[1]-q[0]*p[1] for p,q in zip(path,path[1:]+path[:1]))/2
        if abs(area)<6:continue
        # Split closed loops into two arcs before simplification.
        split=max(range(len(path)),key=lambda i:(path[i][0]-first[0])**2+(path[i][1]-first[1])**2)
        reduced=simplify(path[:split+1])[:-1]+simplify(path[split:]+[first])[:-1]
        result.append([[x-ORIGIN[0],y-ORIGIN[1]] for x,y in reduced])
    return result
out={'width':1320,'height':440,'source_sha256':hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
    'ink':contours(ink),'mage':contours(purple),'purple':'#8818ff',
    'stem':{'x':1161,'y':18,'width':49,'height':256},
    'wheel':{'center_x':1185.5,'top':34,'bottom':258,'width':28,'height':106,'rest_y':100}}
(ROOT/'assets/brand/lockup-v2.paths.json').write_text(json.dumps(out,separators=(',',':'))+'\n')
print(json.dumps({'paths':{k:len(out[k]) for k in ('ink','mage')},'points':{k:sum(map(len,out[k])) for k in ('ink','mage')},'source_sha256':out['source_sha256']},indent=2))

