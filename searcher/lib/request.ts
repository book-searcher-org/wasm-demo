type Header = {name: string, value: string};

function parse_headers(xhr: XMLHttpRequest): Header[] {
    const headers = xhr.getAllResponseHeaders().toLowerCase();
    const arr = headers.trim().split(/[\r\n]+/);
    const headers_arr: Header[] = [];
    arr.forEach((line) => {
      const parts = line.split(': ');
      const name = parts.shift();
      const value = parts.join(': ');
      headers_arr.push({name: name!, value});
    });
    return headers_arr;
}

export function request(method: string, url: string, headers: {name: string, value: string}[]) {
    console.log("[REQUEST] "+ method+" "+ url);

    var xhr = new XMLHttpRequest();
    xhr.overrideMimeType('text/plain; charset=x-user-defined');
    xhr.open(method, url, false);
    if (headers !== undefined) {
        headers.forEach((header) => {
          xhr.setRequestHeader(header.name, header.value)
        });
    }
    xhr.send(null);
    if (xhr.status >= 200 && xhr.status < 300) {
        let resp =  { data: Uint8Array.from(xhr.response, c => c.toString().charCodeAt(0)), headers: parse_headers(xhr) };
        console.log(resp);
        return resp;
    } else {
        throw {
            status: xhr.status,
            status_text: xhr.statusText
        };
    }
}
