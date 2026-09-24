def strip_abs_path:
  gsub("path\\+file://[^#]*"; "path+file://.")
  | gsub("file:///[^#]*"; "file://.")
  | gsub(" bin-target-[0-9]+"; "");

def clean_purl:
  if test("\\?download_url=") then
    sub("\\?download_url=[^&]*"; "")
  else . end;

def make_purl:
  "pkg:cargo/" + .name + "@" + .version;

walk(
  if type == "string" then
    strip_abs_path | clean_purl
  else . end
)

| if .metadata.component.name == "siligpu" then
    .metadata.component.purl = (.metadata.component | make_purl)
    | .metadata.component["bom-ref"] = (.metadata.component | make_purl)
    | .metadata.component.components = (.metadata.component.components // [] | map(
        if .name == "siligpu" then
          .purl = (. | make_purl)
          | .["bom-ref"] = (. | make_purl) + (
            if .type == "library" then "-lib"
            elif .type == "application" then "-app"
            else "" end
          )
        else . end
      ))
  else . end

| .dependencies = (.dependencies // [] | map(
    if (.ref | startswith("path+file://.")) then
      .ref = "pkg:cargo/siligpu@1.0.0"
    else . end
    | .dependsOn = (.dependsOn // [] | map(
        if startswith("path+file://.") then
          "pkg:cargo/siligpu@1.0.0"
        else . end
      ))
  ))