"""Land of the Earth avatar: Natural Earth 1:110m land (public domain,
https://www.naturalearthdata.com), orthographic view centred on Europe and
Africa. Usage: python3 earth-ortho.py ne_110m_land.geojson 60 > earth-land.txt
"""
import json,math,sys
lon0,lat0=math.radians(15),math.radians(22); R=float(sys.argv[2]); cx=cy=128
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
        if len(s)<3: continue
        out.append("M"+" L".join(f"{x:.1f} {y:.1f}" for x,y in s)+"z")
print("".join(out))
