"""Land of the Earth avatar: Natural Earth 1:110m land (public domain,
https://www.naturalearthdata.com), orthographic view centred on Europe and
Africa. Usage: python3 earth-ortho.py ne_110m_land.geojson 60 4 60 > earth-land.txt
(radius, simplification tolerance and smallest island area, in pixels)
"""
import json,math,sys
lon0,lat0=math.radians(15),math.radians(22); R=float(sys.argv[2]); cx=cy=128
# Simplification tolerance and smallest kept area, in pixels: schematic land
TOL=float(sys.argv[3]) if len(sys.argv)>3 else 4.0; MIN_AREA=float(sys.argv[4]) if len(sys.argv)>4 else 60.0
def dp(pts,tol):
    if len(pts)<3: return pts
    (x0,y0),(x1,y1)=pts[0],pts[-1]; L=math.hypot(x1-x0,y1-y0)
    i,dm=0,0
    for k in range(1,len(pts)-1):
        x,y=pts[k]
        # distance to the chord, or to its start when the ring is closed
        dd=abs((x1-x0)*(y0-y)-(x0-x)*(y1-y0))/L if L>1e-6 else math.hypot(x-x0,y-y0)
        if dd>dm: i,dm=k,dd
    if dm<=tol: return [pts[0],pts[-1]]
    return dp(pts[:i+1],tol)[:-1]+dp(pts[i:],tol)
def area(p): return abs(sum(p[k][0]*p[k-1][1]-p[k-1][0]*p[k][1] for k in range(len(p))))/2
g=json.load(open(sys.argv[1]))
def proj(lon,lat):
    l,p=math.radians(lon),math.radians(lat)
    cosc=math.sin(lat0)*math.sin(p)+math.cos(lat0)*math.cos(p)*math.cos(l-lon0)
    x=math.cos(p)*math.sin(l-lon0); y=math.cos(lat0)*math.sin(p)-math.sin(lat0)*math.cos(p)*math.cos(l-lon0)
    if cosc<0:
        n=math.hypot(x,y) or 1; x,y=x/n,y/n
    return cx+R*x, cy-R*y, cosc>=0
out=[]
for f in g['features']:
    geom=f['geometry']; polys=geom['coordinates'] if geom['type']=='MultiPolygon' else [geom['coordinates']]
    for poly in polys:
        ring=poly[0]; pts=[proj(*c) for c in ring]
        if not any(v for _,_,v in pts): continue
        # densify dropping near-duplicates
        s=[];last=None
        for x,y,_ in pts:
            if last and math.hypot(x-last[0],y-last[1])<1.2: continue
            s.append((x,y));last=(x,y)
        s=dp(s+[s[0]],TOL)[:-1]
        if len(s)<3 or area(s)<MIN_AREA: continue
        # Smooth outline: quadratic curves through the midpoints of the edges
        mid=[((s[k][0]+s[k-1][0])/2,(s[k][1]+s[k-1][1])/2) for k in range(len(s))]
        d=f"M{mid[0][0]:.1f} {mid[0][1]:.1f}"
        for k in range(len(s)):
            m=mid[(k+1)%len(s)]
            d+=f"Q{s[k][0]:.1f} {s[k][1]:.1f} {m[0]:.1f} {m[1]:.1f}"
        out.append(d+"z")
print("".join(out))
