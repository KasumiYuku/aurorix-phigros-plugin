# {{title}} · RKS {{rks}}

``` text
官方 RKS  {{official}}
重算 RKS  {{computed}}
成绩总数  {{records}}
{{avgLine}}
满分曲    {{phiCount}} 首
```

{{#each rows}}
{{rank}}. {{title}} [{{difficulty}}] {{constant}}
  {{score}} · {{acc}} · {{grade}} · {{suggest}}
{{/each}}

{{footer}}
