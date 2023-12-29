from libqtile import bar, widget
import subprocess

def get_vpn_status():
    try:
        # Run the curl command and capture the output
        result = subprocess.check_output(['curl', 'https://am.i.mullvad.net/connected']).decode('utf-8').strip()
        return result == 'Connected'
    except subprocess.CalledProcessError:
        # Handle any errors that may occur
        return False

class VPNStatusWidget(widget.TextBox):
    def __init__(self, **config):
        super().__init__('🔒', **config)
        self.update()

    def update(self):
        # Check the VPN status and update the widget text accordingly
        if get_vpn_status():
            self.text = '🔒'  # Locked icon
        else:
            self.text = '🔓'  # Unlocked icon
