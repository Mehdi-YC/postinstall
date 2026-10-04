# Espanso on Wayland (playbook: the ESPANSO section).
#
# services.espanso wires up espanso-wayland (x11Support off) as a user
# service and generates the YAML under ~/.config/espanso from these
# options — the old files in dotfiles/.config/espanso are the source.
{
  services.espanso = {
    enable = true;
    waylandSupport = true;
    x11Support = false;

    # dotfiles/.config/espanso/config/default.yml
    configs.default = {
      keyboard_layout.layout = "fr";
    };

    # dotfiles/.config/espanso/match/*.yml
    matches = {
      base.matches = [
        {
          trigger = ":espanso";
          replace = "Hi there!";
        }
        {
          trigger = ":date";
          replace = "{{mydate}}";
          vars = [
            {
              name = "mydate";
              type = "date";
              params.format = "%m/%d/%Y";
            }
          ];
        }
        {
          trigger = ":shell";
          replace = "{{output}}";
          vars = [
            {
              name = "output";
              type = "shell";
              params.cmd = "echo 'Hello from your shell'";
            }
          ];
        }
      ];

      kostango.matches = [
        {
          trigger = ":ca";
          replace = "#YAHIA CHERIF Mohamed Mahdi add {{today}} : $|$ \n#Notion: \n#End YAHIA CHERIF Mohamed Mahdi {{today}}";
          vars = [
            {
              name = "today";
              type = "date";
              params.format = "%d/%m/%Y";
            }
          ];
        }
        {
          trigger = ":ce";
          replace = "#YAHIA CHERIF Mohamed Mahdi Edit {{today}} : $|$ \n#End YAHIA CHERIF Mohamed Mahdi {{today}}";
          vars = [
            {
              name = "today";
              type = "date";
              params.format = "%d/%m/%Y";
            }
          ];
        }
        {
          trigger = ":date";
          replace = "{{mydate}}";
          vars = [
            {
              name = "mydate";
              type = "date";
              params.format = "%m/%d/%Y";
            }
          ];
        }
        {
          trigger = ":ss";
          replace = ''
            {
            "One__":"badge badge-light-info rounded-pill",
            "Two__":"badge badge-light-success rounded-pill"
            }
          '';
        }
        {
          trigger = ":ia";
          replace = ''
            if record.get("{{form.field}}") and record.get("{{form.field}}").name == "{{form.value}}":return True
            return False
          '';
          vars = [
            {
              name = "form";
              type = "form";
              params.layout = "Field: [[field]] \nValue: [[value]]";
            }
          ];
        }
      ];

      rust.matches = [
        {
          trigger = ":main";
          replace = ''
            fn main() {
                println!("Hello, world!");
            }
          '';
        }
        {
          trigger = ":fn";
          replace = ''
            fn function_name(i: i32) -> i32 {
                // code here
            }
          '';
        }
        {
          trigger = ":struct";
          replace = ''
            struct StructName {
                field: Type,
            }
          '';
        }
        {
          trigger = ":enum";
          replace = ''
            enum EnumName {
                Variant1,
                Variant2,
            }
          '';
        }
        {
          trigger = ":impl";
          replace = ''
            impl StructName {
                fn new() -> Self {
                    // constructor
                }
            }
          '';
        }
        {
          trigger = ":implt";
          replace = ''
            impl Trait for Struct {
                fn method(&self) {
                    // method implementation
                }
            }
          '';
        }
        {
          trigger = ":test";
          replace = ''
            #[cfg(test)]
            mod tests {
                use super::*;

                #[test]
                fn test_example() {
                    assert_eq!(2 + 2, 4);
                }
            }
          '';
        }
      ];
    };
  };
}
