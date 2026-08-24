require "a"
# ^ function.method.builtin

class Car < Vehicle
  # <- keyword
  #    ^ constructor

  def init(id)
    # <- keyword
    # ^ function.method

    @id = id
    # <- property
    #     ^ variable.parameter

    yield
    # <- keyword
    return
    # <- keyword
    next
    # <- keyword
  end

  private
  # ^ function.method.builtin

  public
  # ^ function.method.builtin

  protected
  # ^ function.method.builtin

  private :with_symbol
  # <- function.method.builtin

  private def with_def; end
  # <- function.method.builtin

  def shadowed
    private = 1
    # <- variable
    private
    # <- variable
    private
  end

  def unrelated(acl)
    acl.public
    #   ^ function.method
  end
end
# <- keyword
