(function (root) {


    /*
 *  Object.assign(), described in E6 Section 19.1.2.1
 *
 *  http://www.ecma-international.org/ecma-262/6.0/index.html#sec-object.assign
 */

    if (typeof Object.assign === 'undefined') {
        Object.defineProperty(Object, 'assign', {
            value: function (target) {
                var i, n, j, m, k;
                var source, keys;
                var gotError;
                var pendingError;

                if (target == null) {
                    throw new Exception('target null or undefined');
                }

                for (i = 1, n = arguments.length; i < n; i++) {
                    source = arguments[i];
                    if (source == null) {
                        continue;  // null or undefined
                    }
                    source = Object(source);
                    keys = Object.keys(source);  // enumerable own keys

                    for (j = 0, m = keys.length; j < m; j++) {
                        k = keys[j];
                        try {
                            target[k] = source[k];
                        } catch (e) {
                            if (!gotError) {
                                gotError = true;
                                pendingError = e;
                            }
                        }
                    }
                }

                if (gotError) {
                    throw pendingError;
                }
            }, writable: true, enumerable: false, configurable: true
        });
    }


    root.Package = (function () {
        function Package(name, content) {
            this.name = name;
            this.content = content;
        }
        return Package;
    })();

    root.$package = function (name, content) {
        return new Package(name, content);
    }

    root.$ok = function (packageOrName, content) {
        var pack
        if (packageOrName instanceof Package) {
            pack = packageOrName
        } else {
            pack = new Package(packageOrName, content);
        }
        return {
            type: 'ok',
            package: pack
        }
    }

    root.$err = function (errorOrMsg) {
        var pack
        if (errorOrMsg instanceof Error) {
            pack = errorOrMsg
        } else {
            pack = new Error(errorOrMsg);
        }
        return {
            type: 'err',
            error: pack
        }
    }

    root.$then = function (packageOrName, content) {
        var pack
        if (packageOrName instanceof Package) {
            pack = packageOrName
        } else {
            pack = new Package(packageOrName, content);
        }
        return {
            type: 'then',
            package: pack
        }
    }

    root.$work = function (steps) {
        return {
            type: 'work',
            steps: steps
        }
    }

    root.worktype = new Proxy({}, {
        get: function (target, prop) {
            return function (options) {
                return Object.assign({}, options, {
                    type: prop
                });
            }
        }
    })

});