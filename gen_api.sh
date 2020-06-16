#!/usr/bin/env sh
bindgen ../engine/fonline/API/API_Client.h -o ffi/API_Client.rs_raw -- -x c++ -m32
bindgen ../engine/fonline/API/API_Server.h -o ffi/API_Server.rs_raw -- -x c++ -m32
bindgen ../engine/fonline/API/API_AngelScript.h -o ffi/API_AngelScript.rs_raw -- -x c++ -m32

#regex='extern "C" \{([^}]*)\}'
#rs_raw=`cat ffi/API_Client.rs_raw`
#[[ $rs_raw =~ $regex ]]
#echo "dynamic_ffi!(ClientApi, ${BASH_REMATCH[1]});" > ffi/API_Client.rs
echo "dynamic_ffi!(ClientApi, " > ffi/API_Client.rs
grep -zoP '(?<=extern "C" \{)[^}]*' ffi/API_Client.rs_raw | tr -d '\0' >> ffi/API_Client.rs
echo ");" >> ffi/API_Client.rs

echo "dynamic_ffi!(ServerApi, " > ffi/API_Server.rs
grep -zoP '(?<=extern "C" \{)[^}]*' ffi/API_Server.rs_raw | tr -d '\0' >> ffi/API_Server.rs
echo ");" >> ffi/API_Server.rs

echo "dynamic_ffi!(AngelScriptApi, " > ffi/API_AngelScript.rs
grep -zoP '(?<=extern "C" \{)[^}]*' ffi/API_AngelScript.rs_raw | tr -d '\0' >> ffi/API_AngelScript.rs
echo ");" >> ffi/API_AngelScript.rs

