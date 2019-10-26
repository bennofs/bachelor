SELECT ?node

# obtain all nodes (except value nodes) (item, statements, references)
WITH {
  SELECT ?item ?stmt ?rnode WHERE {
    BIND (wd:Q42 as ?item) .
    
    # collect statements
    OPTIONAL {
      ?item ?ps ?stmt .
      [] wikibase:claim ?ps .
      
      # collect references
      OPTIONAL {
        ?stmt prov:wasDerivedFrom ?rnode .
      }
    }
  } 
} AS %partHelper
WITH {
  SELECT ?part {
    { SELECT DISTINCT (?item as ?part) WHERE { INCLUDE %partHelper FILTER bound(?item) } }
    UNION
    { SELECT DISTINCT (?stmt as ?part) WHERE { INCLUDE %partHelper FILTER bound(?stmt) } }
    UNION
    { SELECT DISTINCT (?rnode as ?part) WHERE { INCLUDE %partHelper FILTER bound(?rnode) } }
  }
} AS %parts

# collect all value nodes for each part
WITH {
  SELECT DISTINCT ?vnode WHERE {
    INCLUDE %parts .
    ?part ?p ?vnode .
    [] wikibase:statementValue|wikibase:statementValueNormalized|wikibase:referenceValue|wikibase:referenceValueNormalized|wikibase:qualifierValue|wikibase:qualifierValueNormalized ?p .
    { { ?vnode a wikibase:QuantityValue } UNION { ?vnode a wikibase:TimeValue } UNION {?vnode a wikibase:GlobecoordinateValue }} . 
  }
} AS %vnodes 

WHERE {
  { SELECT ?node WHERE {
    { SELECT (?part as ?node) WHERE { INCLUDE %parts } }
    UNION
    { SELECT (?vnode as ?node) WHERE { INCLUDE %vnodes } }
  }}
} LIMIT 20