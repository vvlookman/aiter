use std::collections::{HashMap, HashSet};

use geo::{Distance, Euclidean, Geometry};
use geojson::GeoJson;
use rstar::{PointDistance, RTree, RTreeObject, AABB};

use crate::{error::AiterResult, utils::json::json_value_to_string};

pub struct Feature<T> {
    pub geom: Geometry<f64>,
    pub attrs: T,
}

pub fn geojson_to_csv(geojson_str: &str) -> AiterResult<String> {
    let geojson = geojson_str.parse::<GeoJson>()?;

    let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);

    if let GeoJson::FeatureCollection(fc) = geojson {
        let mut headers: HashSet<String> = HashSet::new();
        for feature in &fc.features {
            if let Some(properties) = &feature.properties {
                headers.extend(properties.keys().map(|k| k.to_string()));
            }
        }

        let mut headers: Vec<String> = headers.into_iter().collect();
        headers.sort();
        let _ = wtr.write_record(&headers);

        for feature in &fc.features {
            let mut record = csv::StringRecord::new();

            for field_name in &headers {
                let val: &str = if let Some(property) = &feature.property(field_name) {
                    &json_value_to_string(property)
                } else {
                    ""
                };

                record.push_field(val);
            }

            let _ = wtr.write_record(&record);
        }
    }

    let _ = wtr.flush();

    if let Ok(bytes) = wtr.into_inner() {
        Ok(String::from_utf8_lossy(&bytes).to_string())
    } else {
        Ok(String::new())
    }
}

pub fn geojson_to_rtree(
    geojson_str: &str,
) -> AiterResult<RTree<Feature<HashMap<String, serde_json::Value>>>> {
    let geojson = geojson_str.parse::<GeoJson>()?;

    let mut features: Vec<Feature<HashMap<String, serde_json::Value>>> = vec![];

    if let GeoJson::FeatureCollection(fc) = geojson {
        for feature in &fc.features {
            if let Some(json_geom) = &feature.geometry {
                if let Ok(geom) = TryInto::<Geometry<f64>>::try_into(json_geom) {
                    let mut attrs: HashMap<String, serde_json::Value> = HashMap::new();
                    if let Some(properties) = &feature.properties {
                        for (k, v) in properties {
                            attrs.insert(k.to_string(), v.clone());
                        }
                    }
                    features.push(Feature { geom, attrs });
                }
            }
        }
    }

    Ok(RTree::bulk_load(features))
}

static PLANE: Euclidean = Euclidean {};

impl<T> RTreeObject for Feature<T> {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_points(&points_from_geometry(&self.geom))
    }
}

impl<T> PointDistance for Feature<T> {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let geo_point = geo::Point::new(point[0], point[1]);
        PLANE.distance(&self.geom, &geo_point)
    }
}

fn points_from_geometry(g: &Geometry<f64>) -> Vec<[f64; 2]> {
    match g {
        Geometry::Point(pt) => vec![[pt.x(), pt.y()]],
        Geometry::MultiPoint(mpt) => mpt.iter().map(|pt| [pt.x(), pt.y()]).collect(),
        Geometry::Line(ln) => vec![[ln.start.x, ln.start.y], [ln.end.x, ln.end.y]],
        Geometry::LineString(ls) => points_from_line_string(ls),
        Geometry::MultiLineString(mls) => mls.iter().flat_map(points_from_line_string).collect(),
        Geometry::Polygon(pg) => points_from_line_string(pg.exterior()),
        Geometry::MultiPolygon(mpg) => mpg
            .iter()
            .flat_map(|pg| points_from_line_string(pg.exterior()))
            .collect(),
        Geometry::Rect(rect) => vec![[rect.min().x, rect.min().y], [rect.max().x, rect.max().y]],
        Geometry::Triangle(tr) => points_from_line_string(tr.to_polygon().exterior()),
        Geometry::GeometryCollection(geoms) => {
            geoms.iter().flat_map(points_from_geometry).collect()
        }
    }
}

fn points_from_line_string(ls: &geo::LineString<f64>) -> Vec<[f64; 2]> {
    ls.clone()
        .into_points()
        .iter()
        .map(|pt| [pt.x(), pt.y()])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geojson_to_csv() {
        let geojson = r#"
{
    "type": "FeatureCollection",
    "features": [{
        "type": "Feature",
        "geometry": {
            "type": "Point",
            "coordinates": [125.6, 10.1]
        },
        "properties": {
            "id": 1,
            "name": "Dinagat Islands"
        }
    }]
}
"#;
        assert_eq!(
            geojson_to_csv(geojson).unwrap(),
            "id,name\n1,Dinagat Islands\n"
        );
    }
}
