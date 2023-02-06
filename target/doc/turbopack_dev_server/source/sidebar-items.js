window.SIDEBAR_ITEMS = {"enum":[["ContentSourceContent","The content of a result that is returned by a content source."],["ContentSourceDataFilter","Filter function that describes which information is required."],["ContentSourceResult","The return value of a content source when getting a path. A specificity is attached and when combining results this specificity should be used to order results."]],"mod":[["asset_graph",""],["combined",""],["conditional",""],["headers",""],["lazy_instantiated",""],["query",""],["request",""],["router",""],["source_maps",""],["specificity",""],["static_assets",""]],"struct":[["Body","A request body."],["BodyVc","Vc for [`Body`]"],["Bytes","A wrapper around [hyper::body::Bytes] that implements [Serialize] and [Deserialize]."],["ContentSourceContentVc","Vc for [`ContentSourceContent`]"],["ContentSourceData","Additional info passed to the ContentSource. It was extracted from the http request."],["ContentSourceDataVary","Describes additional information that need to be sent to requests to ContentSource. By sending these information ContentSource responses are cached-keyed by them and they can access them."],["ContentSourceDataVaryVc","Vc for [`ContentSourceDataVary`]"],["ContentSourceDataVc","Vc for [`ContentSourceData`]"],["ContentSourceResultVc","Vc for [`ContentSourceResult`]"],["ContentSourceVc",""],["ContentSources",""],["ContentSourcesVc","Vc for [`ContentSources`]"],["GetContentSourceContentVc",""],["HeaderList","A list of headers arranged as contiguous (name, value) pairs."],["HeaderListVc","Vc for [`HeaderList`]"],["NeededData","Needed data content signals that the content source requires more information in order to serve the request. The held data allows us to partially compute some data, and resume computation after the needed vary data is supplied by the dev server."],["NeededDataVc","Vc for [`NeededData`]"],["NoContentSource","An empty ContentSource implementation that responds with NotFound for every request."],["NoContentSourceVc","Vc for [`NoContentSource`]"],["ProxyResult","The result of proxying a request to another HTTP server."],["ProxyResultVc","Vc for [`ProxyResult`]"],["Rewrite","A rewrite returned from a [ContentSource]. This tells the dev server to update its parsed url, path, and queries with this new information, and any later [NeededData] will receive data out of t these new values."],["RewriteVc","Vc for [`Rewrite`]"],["StaticContent",""],["StaticContentVc","Vc for [`StaticContent`]"]],"trait":[["ContentSource","A source of content that the dev server uses to respond to http requests."],["GetContentSourceContent","A functor to receive the actual content of a content source result."]],"type":[["BodyReadRef","see [turbo_tasks::ReadRef]"],["ContentSourceContentReadRef","see [turbo_tasks::ReadRef]"],["ContentSourceDataReadRef","see [turbo_tasks::ReadRef]"],["ContentSourceDataVaryReadRef","see [turbo_tasks::ReadRef]"],["ContentSourceResultReadRef","see [turbo_tasks::ReadRef]"],["ContentSourcesReadRef","see [turbo_tasks::ReadRef]"],["HeaderListReadRef","see [turbo_tasks::ReadRef]"],["NeededDataReadRef","see [turbo_tasks::ReadRef]"],["NoContentSourceReadRef","see [turbo_tasks::ReadRef]"],["ProxyResultReadRef","see [turbo_tasks::ReadRef]"],["RewriteReadRef","see [turbo_tasks::ReadRef]"],["StaticContentReadRef","see [turbo_tasks::ReadRef]"]]};