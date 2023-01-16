import init, { search } from 'searcher';

init();

function search_click() {
  let query = $("#query").val();
  console.log(query);
  let result = search(query);
  result.forEach(book => {
    console.log(book);
  });

  buildHtmlTable(result, "#result");
}
window.search_click = search_click;


function buildHtmlTable(result, selector) {
  var columns = addAllColumnHeaders(result, selector);

  for (var i = 0; i < result.length; i++) {
    var row$ = $('<tr/>');
    for (var colIndex = 0; colIndex < columns.length; colIndex++) {
      var cellValue = result[i][columns[colIndex]];
      if (cellValue == null) cellValue = "";
      row$.append($('<td/>').html(cellValue));
    }
    $(selector).append(row$);
  }
}

function addAllColumnHeaders(myList, selector) {
  var columnSet = [];
  var headerTr$ = $('<tr/>');

  for (var i = 0; i < myList.length; i++) {
    var rowHash = myList[i];
    for (var key in rowHash) {
      if ($.inArray(key, columnSet) == -1) {
        columnSet.push(key);
        headerTr$.append($('<th/>').html(key));
      }
    }
  }
  $(selector).append(headerTr$);

  return columnSet;
}